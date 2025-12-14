use crate::bus::{bus::EventBus, event::Event};
use tokio::time::Duration;

pub async fn run(bus: EventBus) {
    let mut rx = bus.subscribe();

    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(1))=>{
                bus.publish(Event::CameraFrameReady);
            }
            msg = rx.recv()=>{
                if matches!(msg, Ok(Event::Shutdown)) {
                    println!("Camera node shutting down");
                    break;
                }
            }
        }
    }
}
