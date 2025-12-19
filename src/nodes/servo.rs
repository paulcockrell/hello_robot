use std::{sync::mpsc, time::Duration};

use crate::{
    bus::{event::Event, event_bus::EventBus},
    hal::servo::Servo,
};

// 🌐 1. Async / control world (Tokio)
// * Listens to bus_rx (your event bus)
// * Decides what should happen
// * Sends intent (angle, speed, etc.)
// * Never touches hardware
//
// ⚙️ 2. Blocking / hardware world (OS thread)
// * Owns the servo
// * Owns timing
// * Waits for commands
// * Talks directly to GPIO / PWM
// * They communicate through a standard channel.
// * In short: Async code decides what should happen; blocking code decides when and how.
//
// async task        blocking task
//  │                  │
//  │   send intent    │
//  └──────────────▶   │
//                     │ waits (blocking OK)
//                     │ controls hardware

pub async fn run(bus: EventBus) {
    let mut bus_rx = bus.subscribe();

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
