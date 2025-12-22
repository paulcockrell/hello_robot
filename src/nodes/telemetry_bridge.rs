use serde::Serialize;
use tokio::sync::broadcast;

use crate::{
    AppState,
    bus::event::{Event, Ldr, Ultrasound},
};

#[derive(Serialize, Clone)]
pub enum Telemetry {
    Ultrasound(Ultrasound),
    Ldr(Ldr),
}

pub type TelemetryTx = broadcast::Sender<Telemetry>;

pub async fn run(app_state: AppState) {
    let mut rx = app_state.bus.subscribe();

    while let Ok(event) = rx.recv().await {
        match event {
            Event::Ultrasound(ultrasound) => {
                let _ = app_state
                    .telemetry_tx
                    .send(Telemetry::Ultrasound(ultrasound));
            }
            Event::Ldr(ldr) => {
                let _ = app_state.telemetry_tx.send(Telemetry::Ldr(ldr));
            }
            _ => {}
        }
    }
}
