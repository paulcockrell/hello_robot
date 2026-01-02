use serde::Serialize;
use tokio::sync::broadcast;

use crate::{
    AppState,
    bus::event::{Event, Ldr, Led, Ultrasound},
};

#[derive(Serialize, Clone)]
pub enum Telemetry {
    Ultrasound(Ultrasound),
    Ldr(Ldr),
    Led(Led),
}

pub type TelemetryTx = broadcast::Sender<Telemetry>;

pub async fn run(app_state: AppState) {
    let mut bus_rx = app_state.bus.subscribe();

    while let Ok(event) = bus_rx.recv().await {
        match event {
            Event::Ultrasound(ultrasound) => {
                let _ = app_state
                    .telemetry_tx
                    .send(Telemetry::Ultrasound(ultrasound));
            }
            Event::Ldr(ldr) => {
                let _ = app_state.telemetry_tx.send(Telemetry::Ldr(ldr));
            }
            Event::Led(led) => {
                let _ = app_state.telemetry_tx.send(Telemetry::Led(led));
            }
            Event::Shutdown => {
                println!("Telemetry node shutting down");
                break;
            }
            _ => {}
        }
    }
}
