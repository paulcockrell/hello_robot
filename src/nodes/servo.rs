use std::{sync::mpsc, time::Duration};

use crate::{
    bus::{
        event::{Event, ServoCommand},
        event_bus::EventBus,
    },
    hal::servo::Servo,
};

// Async
// * Listens to bus_rx
// * Decides what should happen
// * Sends intent (new servo angle)
// * Never touches hardware
//
// Blocking
// * Owns the servo
// * Owns timing
// * Waits for commands
// * Talks directly to GPIO / PWM
//
// async task        blocking task
//  │                  │
//  │   send intent    │
//  └──────────────▶   │
//                     │ waits (blocking OK)
//                     │ controls hardware
pub async fn run(bus: EventBus) {
    let mut bus_rx = bus.subscribe();
    let bus_tx = bus.clone();

    // Channel between async world and blocking servo thread
    let (tx, rx) = mpsc::channel::<u8>();

    // === Blocking hardware thread ===
    let servo_task = tokio::task::spawn_blocking(move || {
        let mut servo = Servo::new().expect("Servo init failed");
        let mut last_angle = None;

        loop {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(angle) => {
                    // Only move if changed
                    if last_angle != Some(angle) {
                        let _ = servo.set_angle(angle);
                        last_angle = Some(angle);

                        bus_tx.publish(Event::Servo(ServoCommand { angle }));
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // idle tick, do nothing
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // async side dropped sender so shutdown
                    break;
                }
            }
        }
    });

    // === Async control loop ===
    loop {
        match bus_rx.recv().await {
            Ok(Event::ServoCommand(cmd)) => {
                let _ = tx.send(cmd.angle);
            }
            Ok(Event::Shutdown) => {
                println!("Servo node shutting down");
                break;
            }
            Err(_) => break, // bus closed
            _ => {}
        }
    }

    // Drop tx -> unblock blocking thread
    drop(tx);
    let _ = servo_task.await;
}
