mod hal;
mod state;

use hal::camera::Camera;
use hal::ldr::Ldr;
use hal::motor::Motor;
use hal::neopixel::Neopixel;
use hal::servo::Servo;
use hal::ultrasound::UltrasoundSensor;

use opencv::core::MatTraitConst;
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, Ordering};

use maud::{DOCTYPE, Markup, html};
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::task;
use tokio::time::Duration;

use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use axum_htmx::HxResponseTrigger;
use state::RobotState;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

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

    let state = Arc::new(Mutex::new(RobotState::default()));

    {
        println!("Starting LDR thread");

        let shutdown = shutdown.clone();
        let state = Arc::clone(&state);
        let ldr = Ldr::new(19, 16, 20).unwrap();

        task::spawn_blocking(move || {
            let mut last_reading: (u8, u8, u8) = (0, 0, 0);

            while !shutdown.load(Ordering::SeqCst) {
                let readings = ldr.readings();

                if readings != last_reading {
                    last_reading = readings;

                    let (l_val, m_val, r_val) = readings;
                    let mut st = state.lock().unwrap();
                    st.ldr_left = l_val;
                    st.ldr_middle = m_val;
                    st.ldr_right = r_val;
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
        let state = Arc::clone(&state);
        let mut us = UltrasoundSensor::new(11, 8).unwrap();
        let tx = servo_tx.clone();

        tokio::spawn(async move {
            let mut last_avg = 0.0;
            let mut avg = 1.0;

            while !shutdown.load(Ordering::SeqCst) {
                let dist = us.measure_cm().unwrap_or(0);
                let new_value = dist.clamp(0, 100);
                avg = avg * 0.5 + (new_value as f64) * 0.3;

                if avg != last_avg {
                    last_avg = avg;

                    {
                        let mut st = state.lock().unwrap();
                        st.ultrasound = avg;
                    }

                    if let Err(e) = tx.send(Command::Servo { angle: avg as u8 }).await {
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
        println!("Staring Neopixel thread");

        let shutdown = shutdown.clone();
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            let mut neopixel = Neopixel::new().expect("Neopixel failed");

            while !shutdown.load(Ordering::SeqCst) {
                {
                    let mut st = state.lock().unwrap();
                    let (r_val, g_val, b_val) = distance_to_rgb(st.ultrasound);

                    st.neopixel_r = r_val as u8;
                    st.neopixel_g = g_val as u8;
                    st.neopixel_b = b_val;

                    let _ = neopixel.set_pixels(r_val, g_val, b_val, 0);
                }

                std::thread::sleep(Duration::from_millis(250));
            }

            let _ = neopixel.set_pixels(0, 0, 0, 0);

            println!("Exiting Neopixel thread");
        });
    }

    {
        let mut camera = Camera::new().expect("Camera connect failed");
        println!("Camera ready");

        tokio::spawn(async move {
            loop {
                match camera.grayscale() {
                    Ok(gray) => {
                        let size = gray.size();
                        match size {
                            Ok(s) => {
                                if s.width <= 0 {
                                    println!("Captured empty frame, something may be wrong");
                                }
                            }
                            Err(e) => eprintln!("Failed to get frame size: {e}"),
                        }
                    }
                    Err(e) => eprintln!("Camera error: {e}"),
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
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

    let static_files = ServeDir::new("static");

    let app = Router::new()
        .route("/", get(index))
        .nest_service("/static", static_files)
        .nest_service("/camera/frame.jpg", ServeFile::new("/tmp/frame.jpg"))
        .route("/time", get(time))
        .route("/partials/camera", get(partial_camera))
        .route("/partials/sensors", get(partial_sensors))
        .route("/api/motor/forward", post(|| async { "ok" }))
        .route("/api/motor/stop", post(|| async { "ok" }))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Robot UI running at http://0.0.0.0:3000");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");

    println!("Shutting down...");
}

async fn index() -> Html<String> {
    let html =
        std::fs::read_to_string("templates/index.html").expect("missing templates/index.html");

    Html(html)
}

async fn partial_sensors(
    axum::extract::State(state): axum::extract::State<Arc<Mutex<RobotState>>>,
) -> Html<String> {
    let st = state.lock().unwrap();

    let html = format!(
        r#"
        <ul>
            <li><strong>LDR Left:</strong> {}</li>
            <li><strong>LDR Middle:</strong> {}</li>
            <li><strong>LDR Right:</strong> {}</li>
            <li><strong>Ultrasound:</strong> {} cm</li>
            <li><strong>Neopixel RGB:</strong> {}, {}, {}</li>
        </ul>
        "#,
        st.ldr_left,
        st.ldr_middle,
        st.ldr_right,
        st.ultrasound,
        st.neopixel_r,
        st.neopixel_g,
        st.neopixel_b,
    );

    Html(html)
}

async fn time() -> impl IntoResponse {
    chrono::Utc::now().format("%H:%M:%S").to_string()
}

async fn partial_camera() -> impl IntoResponse {
    let ts = chrono::Utc::now().timestamp_millis();
    format!("/camera/frame.jpg?ts={}", ts)
}

// Convert distance to a red-to-green scale for neopixels
fn distance_to_rgb(distance: f64) -> (u8, u8, u8) {
    let d = distance.clamp(0.0, 100.0);
    let t = d / 100.0;

    let red = (255.0 * (1.0 - t)) as u8;
    let green = (255.0 * t) as u8;
    let blue = 0u8;

    (red, green, blue)
}
