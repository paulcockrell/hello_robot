mod hal;
use hal::ldr::Ldr;
use hal::motor::Motor;
use hal::servo::Servo;
use hal::ultrasound::UltrasoundSensor;

use serde::Deserialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use tokio::sync::mpsc;
use tokio::task;
use tokio::time::{Duration, sleep};

#[derive(Deserialize, Debug)]
enum MotorDirection {
    #[serde(rename = "forward")]
    Forward,
    #[serde(rename = "backward")]
    Backward,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "stop")]
    Stop,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum Command {
    #[serde(rename = "motor")]
    Motor {
        direction: MotorDirection,
        speed: u8,
    },

    #[serde(rename = "servo")]
    Servo { angle: u8 },

    #[serde(rename = "led")]
    Led { r: u8, g: u8, b: u8 },

    #[serde(rename = "camera")]
    Camera { command: String },
}

async fn socket_responder(path: &str, command_tx: mpsc::Sender<String>) -> anyhow::Result<()> {
    let _ = std::fs::remove_file(path);
    let listener = UnixListener::bind(path)?;
    println!("Listening on {}", path);

    loop {
        let (mut stream, _) = listener.accept().await?;

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await?;

        let msg = String::from_utf8_lossy(&buf).to_string();
        println!("CMD = {msg}");

        command_tx.send(msg).await?;
    }
}

#[tokio::main]
async fn main() {
    println!("Starting Main thread");

    let shutdown = Arc::new(AtomicBool::new(false));

    let (command_tx, mut command_rx) = mpsc::channel::<String>(32);

    {
        println!("Starting LDR thread");

        let shutdown = shutdown.clone();
        let ldr = Ldr::new(19, 16, 20).unwrap();

        task::spawn_blocking(move || {
            let mut last_reading: (u8, u8, u8) = (0, 0, 0);

            while !shutdown.load(Ordering::SeqCst) {
                let readings = ldr.readings();

                if readings != last_reading {
                    println!("LDR values: {:?}", readings);
                    last_reading = readings;
                }

                std::thread::sleep(Duration::from_millis(200));
            }

            println!("Exiting LDR thread");
        });
    }

    let (servo_tx, mut servo_rx) = mpsc::channel::<Command>(16);

    {
        println!("Starting Servo thread");

        let shutdown = shutdown.clone();
        let mut servo = Servo::new().unwrap();

        let min_pulse: u16 = 150;
        let max_pulse: u16 = 600;

        task::spawn_blocking(move || {
            while !shutdown.load(Ordering::SeqCst) {
                if let Some(cmd) = servo_rx.blocking_recv() {
                    if let Command::Servo { angle } = cmd {
                        let new_angle = if angle <= 180 { angle } else { 0 };
                        let _ = servo.set_angle(new_angle);
                    }
                }
            }

            println!("Exiting Servo thread");
        });
    }

    {
        println!("Starting Ultrasound thread");

        let shutdown = shutdown.clone();
        let mut us = UltrasoundSensor::new(11, 8).unwrap();
        let tx = servo_tx.clone();

        tokio::spawn(async move {
            let mut last_reading = 0u16;

            while !shutdown.load(Ordering::SeqCst) {
                let dist = us.measure_cm().unwrap_or(0);
                let angle = dist.clamp(0, 100) as u8;

                if dist != last_reading {
                    println!("Ultrasound changed: {}", dist);

                    last_reading = dist;

                    if let Err(e) = tx.send(Command::Servo { angle }).await {
                        eprintln!("Servo send failed: {}", e);
                    }
                }

                std::thread::sleep(Duration::from_millis(100));
            }

            println!("Exiting Ultrasound thread");
        });
    }

    let (motor_tx, mut motor_rx) = mpsc::channel::<Command>(16);

    {
        println!("Starting Motor thread");

        let shutdown = shutdown.clone();
        let mut left = Motor::new(26, 21, 4).unwrap();
        let mut right = Motor::new(27, 18, 17).unwrap();

        tokio::spawn(async move {
            while !shutdown.load(Ordering::SeqCst) {
                if let Some(cmd) = motor_rx.recv().await {
                    if let Command::Motor { direction, speed } = cmd {
                        match direction {
                            MotorDirection::Forward => {
                                let _ = left.forward(speed);
                                let _ = right.forward(speed);
                            }
                            MotorDirection::Backward => {
                                let _ = left.backward(speed);
                                let _ = right.backward(speed);
                            }
                            MotorDirection::Left => {
                                let _ = left.backward(speed);
                                let _ = right.forward(speed);
                            }
                            MotorDirection::Right => {
                                let _ = left.forward(speed);
                                let _ = right.backward(speed);
                            }
                            MotorDirection::Stop => {
                                let _ = left.forward(0);
                                let _ = right.forward(0);
                            }
                        }
                    }
                }
            }

            println!("Exiting Motor thread");
        });
    }

    {
        let tx = command_tx.clone();
        tokio::spawn(async move {
            socket_responder("/tmp/robot.sock", tx)
                .await
                .expect("socket failed");
        });
    }

    {
        println!("Starting Command thread");

        let shutdown = shutdown.clone();

        tokio::spawn(async move {
            while !shutdown.load(Ordering::SeqCst) {
                if let Some(raw) = command_rx.recv().await {
                    let cmd: Result<Command, _> = serde_json::from_str(&raw);
                    match cmd {
                        Ok(cmd) => {
                            match &cmd {
                                Command::Motor { .. } => {
                                    motor_tx.send(cmd).await.unwrap();
                                }
                                Command::Servo { .. } => {
                                    servo_tx.send(cmd).await.unwrap();
                                }
                                Command::Led { .. } => {
                                    // TODO: add LED task
                                }
                                Command::Camera { .. } => {
                                    // TODO: camera task
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("invalid JSON command: {e}");
                        }
                    }
                }
            }
            println!("Exiting Command thread");
        });
    }

    {
        let shutdown = shutdown.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            println!("CTRL-C RECEIVED");
            shutdown.store(true, Ordering::SeqCst);
        });
    }

    // Keep main alive
    while !shutdown.load(Ordering::SeqCst) {
        sleep(Duration::from_millis(200)).await;
    }

    println!("Exiting Main thread");
}
