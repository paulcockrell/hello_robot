mod bus;
mod hal;
mod nodes;

use tokio::task::LocalSet;

use crate::{
    bus::{event::Event, event_bus::EventBus},
    hal::camera::CameraState,
};

#[derive(Debug, Clone)]
struct AppState {
    pub bus: EventBus,
    pub camera: CameraState,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    println!("Starting Main thread");

    let app_state = AppState {
        bus: EventBus::new(64),
        camera: CameraState::new(),
    };

    let handles = vec![
        tokio::spawn(nodes::motor::run(app_state.bus.clone())),
        tokio::spawn(nodes::ldr::run(app_state.bus.clone())),
        tokio::spawn(nodes::ultrasound::run(app_state.bus.clone())),
        tokio::spawn(nodes::camera::run(app_state.clone())),
        tokio::spawn(nodes::web::run(app_state.clone())),
    ];

    // Local hardware node
    let local = LocalSet::new();

    local.spawn_local(nodes::leds::run(app_state.bus.clone()));
    local.spawn_local(nodes::servo::run(app_state.bus.clone()));

    local
        .run_until(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to setup CTRL+C handler");

            println!("CTRL-C received. Shutting down.");
            app_state.bus.publish(Event::Shutdown);

            for h in handles {
                let _ = h.await;
            }
        })
        .await;

    println!("Shutdown complete");
}
