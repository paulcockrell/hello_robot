use std::sync::mpsc;

use crate::{
    bus::{
        event::{Event, MotorCommand, MotorDirection},
        event_bus::EventBus,
    },
    hal::motor::Motor,
};

pub async fn run(bus: EventBus) {
    let mut bus_rx = bus.subscribe();

    let (tx, rx) = mpsc::channel::<MotorCommand>();

    let motor_task = tokio::task::spawn_blocking(move || {
        let mut left = Motor::new(26, 21, 4).unwrap();
        let mut right = Motor::new(27, 18, 17).unwrap();

        while let Ok(cmd) = rx.recv() {
            match cmd.direction {
                MotorDirection::Forward => {
                    let _ = left.forward(cmd.speed);
                    let _ = right.forward(cmd.speed);
                }
                MotorDirection::Backward => {
                    let _ = left.backward(cmd.speed);
                    let _ = right.backward(cmd.speed);
                }
                MotorDirection::Left => {
                    let _ = left.backward(cmd.speed);
                    let _ = right.forward(cmd.speed);
                }
                MotorDirection::Right => {
                    let _ = left.forward(cmd.speed);
                    let _ = right.backward(cmd.speed);
                }
                MotorDirection::Stop => {
                    let _ = left.stop();
                    let _ = right.stop();
                }
            }
        }
    });

    loop {
        match bus_rx.recv().await {
            Ok(Event::MotorCommand(cmd)) => {
                let _ = tx.send(cmd);
            }
            Ok(Event::Shutdown) => {
                println!("Motor node shutting down");
                break;
            }
            Err(_) => break,
            _ => {}
        }
    }

    drop(tx);
    let _ = motor_task.await;
}
