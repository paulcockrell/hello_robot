use axum::response::IntoResponse;
use axum::{Router, response::Html, routing::get};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::bus::bus::EventBus;
use crate::bus::event::Event;

pub async fn run(bus: EventBus) {
    let static_files = ServeDir::new("static");

    let app = Router::new()
        .route("/", get(index))
        .nest_service("/static", static_files)
        // .nest_service("/camera/frame.jpg", ServeFile::new("/tmp/frame.jpg"))
        .route("/time", get(time))
        // .route("/partials/camera", get(partial_camera))
        // .route("/partials/sensors", get(partial_sensors))
        // .route("/api/motor/forward", post(motor_forward_handler))
        // .route("/api/motor/stop", post(motor_stop_handler))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Robot UI running at http://0.0.0.0:3000");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(bus))
        .await
        .unwrap();
}

async fn shutdown_signal(bus: EventBus) {
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
