mod bus;
mod hal;
mod nodes;

use tokio::{
    sync::{broadcast, watch},
    task::LocalSet,
};

use crate::{
    bus::{event::Event, event_bus::EventBus},
    hal::camera::CameraState,
    nodes::telemetry_bridge::TelemetryTx,
};

#[derive(Debug, Clone)]
struct AppState {
    pub bus: EventBus,
    pub camera: CameraState,
    pub shutdown: watch::Receiver<()>,
    pub telemetry_tx: TelemetryTx,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    println!("Starting Main thread");

    let bus = EventBus::new(64);
    let shutdown_rx = spawn_shutdown_bridge(bus.clone());
    let (telemetry_tx, _) = broadcast::channel(64);

    let app_state = AppState {
        bus,
        camera: CameraState::new(),
        shutdown: shutdown_rx,
        telemetry_tx,
    };

    let handles = vec![
        tokio::spawn(nodes::motor::run(app_state.bus.clone())),
        tokio::spawn(nodes::ldr::run(app_state.bus.clone())),
        tokio::spawn(nodes::ultrasound::run(app_state.bus.clone())),
        tokio::spawn(nodes::camera::run(app_state.clone())),
        tokio::spawn(nodes::web::run(app_state.clone())),
        tokio::spawn(nodes::telemetry_bridge::run(app_state.clone())),
        tokio::spawn(nodes::behaviour::run(app_state.clone())),
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

fn spawn_shutdown_bridge(bus: EventBus) -> watch::Receiver<()> {
    let (tx, rx) = watch::channel(());

    tokio::spawn(async move {
        let mut bus_rx = bus.subscribe();
        while let Ok(event) = bus_rx.recv().await {
            if matches!(event, Event::Shutdown) {
                let _ = tx.send(());
                break;
            }
        }
    });

    rx
}
