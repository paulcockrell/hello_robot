mod hal;
use hal::ldr::Ldr;
use hal::motor::Motor;
use hal::ultrasound::UltrasoundSensor;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
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
        let ldr_sensor = Ldr::new(19, 16, 20).unwrap();

        task::spawn_blocking(move || {
            loop {
                let (l_val, m_val, r_val) = ldr_sensor.readings();
                let mut s = state.lock().unwrap();

                s.ldr_left_value = l_val;
                s.ldr_middle_value = m_val;
                s.ldr_right_value = r_val;

                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        });
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
        let mut us_sensor = UltrasoundSensor::new(11, 8).unwrap();

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
    // Task D - Motor controller
    {
        let mut motors_left = Motor::new(26, 21, 4).unwrap();
        let mut motors_right = Motor::new(27, 18, 17).unwrap();
        let speed = 100;

        task::spawn_blocking(move || {
            loop {
                let _ = motors_left.forward(speed);
                let _ = motors_right.forward(speed);
                std::thread::sleep(Duration::from_millis(2000));
                let _ = motors_left.backward(speed);
                let _ = motors_right.backward(speed);
                std::thread::sleep(Duration::from_millis(2000));
            }
        });
    }

    //
    // Task E - Global robot logic loop
    //
    loop {
        let s = state.lock().unwrap();
        println!("ROBOT STATE: {:?}", *s);
        drop(s);

        sleep(Duration::from_secs(1)).await;
    }
}
