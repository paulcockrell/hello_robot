use axum::body::{Body, Bytes};
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::extract::{Json, State};
use axum::response::IntoResponse;
use axum::response::Response;
use axum::{Router, response::Html, routing::get, routing::post};
use futures::stream;
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;

use crate::AppState;
use crate::bus::event::{Event, LedCommand, MotorCommand, MotorDirection, ServoCommand};
use crate::bus::event_bus::EventBus;
use crate::nodes::telemetry_bridge::TelemetryTx;

#[derive(Debug, Deserialize)]
struct WebCommand {
    action: String,
}

pub async fn run(app_state: AppState) {
    let static_files = ServeDir::new("static");

    let app = Router::new()
        .route("/", get(index))
        .nest_service("/static", static_files)
        .nest_service("/camera/frame.jpg", ServeFile::new("/tmp/frame.jpg"))
        .route("/time", get(time))
        .route("/partials/camera", get(partial_camera))
        .route("/camera/frame.mjpeg", get(mjpeg_handler))
        .route("/partials/sensors", get(partial_sensors))
        .route("/api/motor", post(motor_command))
        .route("/api/servo", post(servo_command))
        .route("/api/leds", post(led_command))
        .route("/ws", get(ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(app_state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Robot UI running at http://0.0.0.0:3000");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(app_state.bus.into()))
        .await
        .unwrap();
}

async fn shutdown_signal(bus: Arc<EventBus>) {
    let mut rx = bus.subscribe();

    loop {
        if let Ok(Event::Shutdown) = rx.recv().await {
            println!("Web node shutting down");
            break;
        }
    }
}

async fn index() -> Html<String> {
    let html =
        std::fs::read_to_string("templates/index.html").expect("missing templates/index.html");

    Html(html)
}

async fn time() -> impl IntoResponse {
    chrono::Utc::now().format("%H:%M:%S").to_string()
}

// TODO convert this to read sensor data from bus and publish over websockets
async fn partial_sensors() -> Html<String> {
    let ldr_left = 0;
    let ldr_middle = 0;
    let ldr_right = 0;
    let ultrasound = 0;
    let neopixel_r = 0;
    let neopixel_g = 0;
    let neopixel_b = 0;

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
        ldr_left, ldr_middle, ldr_right, ultrasound, neopixel_r, neopixel_g, neopixel_b,
    );

    Html(html)
}

async fn partial_camera() -> impl IntoResponse {
    let ts = chrono::Utc::now().timestamp_millis();
    format!("/camera/frame.jpg?ts={}", ts)
}

async fn motor_command(
    State(app_state): State<AppState>,
    Json(payload): Json<WebCommand>,
) -> impl IntoResponse {
    println!("Received motor command {:?}", payload);

    match payload.action.as_str() {
        "motor.forward" => motor_foreward_handler(app_state),
        "motor.backward" => motor_backward_handler(app_state),
        "motor.left" => motor_left_handler(app_state),
        "motor.right" => motor_right_handler(app_state),
        "motor.stop" => motor_stop_handler(app_state),
        _ => println!("Unknown command"),
    }

    "Ok"
}

async fn servo_command(
    State(app_state): State<AppState>,
    Json(payload): Json<WebCommand>,
) -> impl IntoResponse {
    println!("Received servo command {:?}", payload);

    match payload.action.as_str() {
        "servo.start" => servo_start_handler(app_state),
        "servo.increment" => servo_increment_handler(app_state),
        "servo.decrement" => servo_decrement_handler(app_state),
        "servo.end" => servo_end_handler(app_state),
        _ => println!("Unknown command"),
    }

    "Ok"
}

async fn led_command(
    State(app_state): State<AppState>,
    Json(payload): Json<WebCommand>,
) -> impl IntoResponse {
    println!("Received leds command {:?}", payload);

    match payload.action.as_str() {
        "leds.increment" => led_brightness_handler(app_state, 5),
        "leds.decrement" => led_brightness_handler(app_state, -5),
        _ => println!("Unknown command"),
    }

    "Ok"
}

fn motor_foreward_handler(app_state: AppState) {
    let cmd = MotorCommand {
        direction: MotorDirection::Forward,
        speed: 100,
    };

    app_state.bus.publish(Event::MotorCommand(cmd));
}

fn motor_backward_handler(app_state: AppState) {
    let cmd = MotorCommand {
        direction: MotorDirection::Backward,
        speed: 90,
    };

    app_state.bus.publish(Event::MotorCommand(cmd));
}

fn motor_left_handler(app_state: AppState) {
    let cmd = MotorCommand {
        direction: MotorDirection::Left,
        speed: 100,
    };

    app_state.bus.publish(Event::MotorCommand(cmd));
}

fn motor_right_handler(app_state: AppState) {
    let cmd = MotorCommand {
        direction: MotorDirection::Right,
        speed: 100,
    };

    app_state.bus.publish(Event::MotorCommand(cmd));
}

fn motor_stop_handler(app_state: AppState) {
    let cmd = MotorCommand {
        direction: MotorDirection::Stop,
        speed: 0,
    };

    app_state.bus.publish(Event::MotorCommand(cmd));
}

fn servo_end_handler(app_state: AppState) {
    let cmd = ServoCommand { angle: 170 };

    app_state.bus.publish(Event::ServoCommand(cmd));
}

fn servo_increment_handler(app_state: AppState) {
    let cmd = ServoCommand { angle: 25 };

    app_state.bus.publish(Event::ServoCommand(cmd));
}

fn servo_decrement_handler(app_state: AppState) {
    let cmd = ServoCommand { angle: 75 };

    app_state.bus.publish(Event::ServoCommand(cmd));
}

fn servo_start_handler(app_state: AppState) {
    let cmd = ServoCommand { angle: 10 };

    app_state.bus.publish(Event::ServoCommand(cmd));
}

fn led_brightness_handler(app_state: AppState, value: i8) {
    let cmd = LedCommand { brightness: value };

    app_state.bus.publish(Event::LedCommand(cmd));
}

async fn mjpeg_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    let stream = stream::unfold((), move |_| {
        let frame = app_state.camera.latest_frame.clone();
        let mut shutdown = app_state.shutdown.clone();

        async move {
            tokio::select! {
                _=shutdown.changed()=>{
                    println!("MJPEG stream shutting down");
                    None
                }

                _ = sleep(Duration::from_millis(50)) => {
                    let jpeg = {
                        let guard = frame.lock().unwrap();
                        if guard.is_empty() {
                            None
                        } else {
                            Some(guard.clone())
                        }
                    }?;

                    let mut chunk = Vec::new();
                    chunk.extend_from_slice(b"--frame\r\n");
                    chunk.extend_from_slice(b"Content-Type: image/jpeg\r\n");
                    chunk.extend_from_slice(
                        format!("Content-Length: {}\r\n\r\n", jpeg.len()).as_bytes(),
                    );
                    chunk.extend_from_slice(&jpeg);
                    chunk.extend_from_slice(b"\r\n");

                    Some((Ok::<Bytes, Infallible>(Bytes::from(chunk)), ()))
                }
            }
        }
    });

    Response::builder()
        .header("Content-Type", "multipart/x-mixed-replace; boundary=frame")
        .body(Body::from_stream(stream))
        .unwrap()
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, app_state.telemetry_tx))
}

async fn handle_socket(mut socket: WebSocket, telemetry_tx: TelemetryTx) {
    let mut rx = telemetry_tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        let json = serde_json::to_string(&msg).unwrap();

        if socket.send(json.into()).await.is_err() {
            break; // client disconnected
        }
    }
}
