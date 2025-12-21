use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Router, response::Html, routing::get, routing::post};
use futures::stream;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;

use crate::AppState;
use crate::bus::event::{Event, MotorCommand, MotorDirection, ServoCommand};
use crate::bus::event_bus::EventBus;

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
        .route("/api/motor/forward", post(motor_forward_handler))
        .route("/api/motor/backward", post(motor_backward_handler))
        .route("/api/motor/left", post(motor_left_handler))
        .route("/api/motor/right", post(motor_right_handler))
        .route("/api/motor/stop", post(motor_stop_handler))
        .route("/api/servo/up", post(servo_up_handler))
        .route("/api/servo/down", post(servo_down_handler))
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

async fn motor_forward_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    println!("Received forward command");

    let cmd = MotorCommand {
        direction: MotorDirection::Forward,
        speed: 100,
    };

    app_state.bus.publish(Event::MotorCommand(cmd));

    "Forward"
}

async fn motor_backward_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    println!("Received backward command");

    let cmd = MotorCommand {
        direction: MotorDirection::Backward,
        speed: 90,
    };

    app_state.bus.publish(Event::MotorCommand(cmd));

    "Backward"
}

async fn motor_left_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    println!("Received turn left command");

    let cmd = MotorCommand {
        direction: MotorDirection::Left,
        speed: 100,
    };

    app_state.bus.publish(Event::MotorCommand(cmd));

    "Left"
}

async fn motor_right_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    println!("Received turn right command");

    let cmd = MotorCommand {
        direction: MotorDirection::Right,
        speed: 100,
    };

    app_state.bus.publish(Event::MotorCommand(cmd));

    "Right"
}

async fn motor_stop_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    println!("Received stop command");

    let cmd = MotorCommand {
        direction: MotorDirection::Stop,
        speed: 0,
    };

    app_state.bus.publish(Event::MotorCommand(cmd));

    "Stop"
}

async fn servo_up_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    println!("Received servo up command");

    let cmd = ServoCommand { angle: 170 };

    app_state.bus.publish(Event::ServoCommand(cmd));

    "Up"
}

async fn servo_down_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    println!("Received servo down command");

    let cmd = ServoCommand { angle: 10 };

    app_state.bus.publish(Event::ServoCommand(cmd));

    "Down"
}

async fn mjpeg_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    let stream = stream::unfold((), move |_| {
        let frame = app_state.camera.latest_frame.clone();

        async move {
            let jpeg_opt = {
                let guard = frame.lock().unwrap();
                if guard.is_empty() {
                    None
                } else {
                    Some(guard.clone())
                }
            };

            if let Some(jpeg) = jpeg_opt {
                let mut chunk = Vec::new();

                chunk.extend_from_slice(b"--frame\r\n");
                chunk.extend_from_slice(b"Content-Type: image/jpeg\r\n");
                chunk.extend_from_slice(
                    format!("Content-Length: {}\r\n\r\n", jpeg.len()).as_bytes(),
                );
                chunk.extend_from_slice(&jpeg);
                chunk.extend_from_slice(b"\r\n");

                sleep(Duration::from_millis(100)).await;

                Some((Ok(Bytes::from(chunk)), ()))
            } else {
                sleep(Duration::from_millis(100)).await;
                Some((Ok::<Bytes, Infallible>(Bytes::new()), ()))
            }
        }
    });

    axum::response::Response::builder()
        .header("Content-Type", "multipart/x-mixed-replace; boundary=frame")
        .body(Body::from_stream(stream))
        .unwrap()
}
