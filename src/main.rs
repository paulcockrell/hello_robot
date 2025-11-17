use rppal::gpio::{Gpio, InputPin, Level, OutputPin};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use tokio::task;
use tokio::time::{Duration, sleep};

#[derive(Debug, Deserialize)]
struct RobotState {
    ldr_left_value: u8,
    ldr_middle_value: u8,
    ldr_right_value: u8,
    ultrasound_distance: u16,
    last_cmd: String,
}

struct UltrasoundSensor {
    trig: OutputPin,
    echo: InputPin,
    max_wait: Duration,
    speed_of_sound: f32,
}

impl UltrasoundSensor {
    pub fn new(trig_pin: u8, echo_pin: u8) -> Self {
        let gpio = Gpio::new().unwrap();

        Self {
            trig: gpio.get(trig_pin).unwrap().into_output_low(),
            echo: gpio.get(echo_pin).unwrap().into_input_pulldown(),
            max_wait: Duration::from_micros(25_000), // 25ms = max sensor timeout
            speed_of_sound: 343.0,                   // m/s @ ~20 deg c
        }
    }

    fn measure_cm(&mut self) -> Option<u16> {
        // Trigger pulse
        self.trig.set_low();
        std::thread::sleep(Duration::from_micros(2));

        self.trig.set_high();
        std::thread::sleep(Duration::from_micros(10));
        self.trig.set_low();

        // Wait for echo to go high
        let start_wait = Instant::now();
        while self.echo.read() == Level::Low {
            if start_wait.elapsed() > self.max_wait {
                return None; // no wait
            }
        }

        // Echo High, start timing pulse
        let pulse_start = Instant::now();
        while self.echo.read() == Level::High {
            if pulse_start.elapsed() > self.max_wait {
                return None;
            }
        }

        let pulse_duration = pulse_start.elapsed();

        // Convert pulse time to distance
        let secs = pulse_duration.as_secs_f32();

        let distance_cm = (secs * self.speed_of_sound * 100.0) / 2.0;

        return Some(distance_cm as u16);
    }
}

fn ldr(state: Arc<Mutex<RobotState>>) {
    let l_pin = Gpio::new().unwrap().get(19).unwrap().into_input();
    let m_pin = Gpio::new().unwrap().get(16).unwrap().into_input();
    let r_pin = Gpio::new().unwrap().get(20).unwrap().into_input();

    loop {
        let l_pin_level = l_pin.read();
        let m_pin_level = m_pin.read();
        let r_pin_level = r_pin.read();

        {
            let mut s = state.lock().unwrap();
            s.ldr_left_value = l_pin_level as u8;
            s.ldr_middle_value = m_pin_level as u8;
            s.ldr_right_value = r_pin_level as u8;
        }

        println!(
            "LDR left: {:?}, middle: {:?}, right: {:?}",
            l_pin_level, m_pin_level, r_pin_level
        );

        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

async fn socket_responder(sock_path: &str, state: Arc<Mutex<RobotState>>) -> anyhow::Result<()> {
    let _ = std::fs::remove_file(sock_path);

    let listener = UnixListener::bind(sock_path)?;
    println!("Listening on Unix socket {}", sock_path);

    loop {
        let (mut stream, _) = listener.accept().await?;

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await?;

        let msg = String::from_utf8_lossy(&buf).to_string();

        {
            let mut s = state.lock().unwrap();
            s.last_cmd = msg.clone();
        }

        println!("CMD = {}", msg);
    }
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(RobotState {
        ldr_left_value: 0,
        ldr_middle_value: 0,
        ldr_right_value: 0,
        ultrasound_distance: 0,
        last_cmd: "none".into(),
    }));

    //
    // Task A - LDR polling
    //
    {
        let state = Arc::clone(&state);
        task::spawn_blocking(move || ldr(state));
    }

    //
    // Task B - Unix Socket responder
    // Send comands example: echo '{"cmd":"hello"}' | socat - UNIX-CONNECT:/tmp/robot.sock
    //
    {
        let state = Arc::clone(&state);
        let sock_path = "/tmp/robot.sock";
        task::spawn(async move {
            socket_responder(sock_path, state).await.unwrap();
        });
    }

    //
    // Task C - Ultrasound sensor
    //
    {
        let state = Arc::clone(&state);
        let mut us_sensor = UltrasoundSensor::new(11, 8);

        task::spawn_blocking(move || {
            loop {
                {
                    let mut s = state.lock().unwrap();
                    s.ultrasound_distance = us_sensor.measure_cm().unwrap_or(0);
                }

                std::thread::sleep(Duration::from_millis(60));
            }
        });
    }

    //
    // Task D - Global robot logic loop
    //
    loop {
        let s = state.lock().unwrap();
        println!("ROBOT STATE: {:?}", *s);
        drop(s);

        sleep(Duration::from_secs(1)).await;
    }
}
