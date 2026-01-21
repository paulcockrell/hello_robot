use std::time::Duration;

use rand::seq::IndexedRandom;

use crate::AppState;
use crate::bus::event::{Event, Mode, MotorCommand, MotorDirection};

pub async fn run(app_state: AppState) {
    let mut bus_rx = app_state.bus.subscribe();
    let bus_tx = app_state.bus.clone();

    let mut mode = Mode::Manual;
    let mut last_distance = 999.9;
    let mut tick = tokio::time::interval(Duration::from_millis(200));
    let mut new_intent: Option<MotorDirection> = Some(MotorDirection::Forward);
    let mut last_intent: Option<MotorDirection> = None;

    loop {
        tokio::select! {
            Ok(event)=bus_rx.recv() => {
                match event {
                    Event::ModeCommand(new_mode) =>  {
                        mode = new_mode.mode;

                        if mode == Mode::Manual {
                            // Reset intents
                            new_intent = Some(MotorDirection::Forward);
                            last_intent = Some(MotorDirection::Forward);

                            // Issue all stop
                            let cmd = MotorCommand {
                                direction: MotorDirection::Stop,
                                speed: 0,
                            };

                            bus_tx.publish(Event::MotorCommand(cmd));
                        }

                        println!("Mode changed to {:?}", mode);
                    },
                    Event::Ultrasound(ultrasound) => last_distance = ultrasound.distance,
                    _ => {}
                }
            }
            _ = tick.tick() => {
                if mode == Mode::Automatic {
                    if last_distance < 10.0 {
                        if last_intent.as_ref() == Some(&MotorDirection::Forward) {
                            new_intent = Some(random_avoidance_intent());
                        } else {
                            new_intent = last_intent.clone();
                        }
                    } else {
                        new_intent = Some(MotorDirection::Forward);
                    }

                    if new_intent.as_ref() != last_intent.as_ref() {
                        if let Some(intent) = new_intent.as_ref() {
                            let cmd = MotorCommand {
                                direction: intent.clone(),
                                speed: 100,
                            };

                            bus_tx.publish(Event::MotorCommand(cmd));

                            println!("[AUTO] New intent selected {:?}", intent);
                        }

                        last_intent = new_intent.clone();
                    }
                }
            }
        }
    }
}

fn random_avoidance_intent() -> MotorDirection {
    let intents = [
        MotorDirection::Left,
        MotorDirection::Right,
        MotorDirection::Backward,
    ];
    let mut rng = rand::rng();

    intents.choose(&mut rng).unwrap().clone()
}
