mod bus;
mod hal;
mod nodes;

use tokio::task::LocalSet;

use crate::bus::{event::Event, event_bus::EventBus};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    println!("Starting Main thread");

    let bus = EventBus::new(64);

    let handles = vec![
        tokio::spawn(nodes::motor::run(bus.clone())),
        tokio::spawn(nodes::ldr::run(bus.clone())),
        tokio::spawn(nodes::ultrasound::run(bus.clone())),
        tokio::spawn(nodes::camera::run(bus.clone())),
        tokio::spawn(nodes::web::run(bus.clone())),
    ];

    // Local hardware node
    let local = LocalSet::new();

    local.spawn_local(nodes::leds::run(bus.clone()));
    local.spawn_local(nodes::servo::run(bus.clone()));

    local
        .run_until(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to setup CTRL+C handler");

            println!("CTRL-C received. Shutting down.");
            bus.publish(Event::Shutdown);

            for h in handles {
                let _ = h.await;
            }
        })
        .await;

    println!("Shutdown complete");
}
