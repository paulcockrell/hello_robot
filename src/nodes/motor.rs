use crate::bus::{bus::EventBus, event::Event};

pub async fn run(bus: EventBus) {
    let mut rx = bus.subscribe();

    loop {
        match rx.recv().await {
            Ok(Event::MotorCommand(cmd)) => {
                // drive motors
            }
            Ok(Event::Shutdown) => {
                println!("Motor node shutting down");
                break;
            }
            _ => {}
        }
    }
}
