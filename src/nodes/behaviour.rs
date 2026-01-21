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
    let mut last_intent: Option<MotorDirection> = None;

    loop {
        tokio::select! {
            Ok(event)=bus_rx.recv() => {
                match event {
                    Event::ModeCommand(new_mode) =>  {
                        mode = new_mode.mode;

                        match mode {
                            Mode::Manual => {
                                // Reset auto intents
                                last_intent = None;

                                // Issue all stop
                                let cmd = MotorCommand {
                                    direction: MotorDirection::Stop,
                                    speed: 0,
                                };

                                bus_tx.publish(Event::MotorCommand(cmd));
                            }

                            Mode::Automatic => {
                                // Reset auto intents
                                last_intent = None;

                                println!("[auto] behaviour reset");
                            }
                        }

                        println!("Mode changed to {:?}", mode);
                    },
                    Event::Ultrasound(ultrasound) => last_distance = ultrasound.distance,
                    _ => {}
                }
            }
            _ = tick.tick() => {
                if mode == Mode::Automatic {
                    let new_intent = if last_distance < 10.0 {
                        if last_intent.as_ref() == Some(&MotorDirection::Forward) {
                            Some(random_avoidance_intent())
                        } else {
                            last_intent.clone()
                        }
                    } else {
                        Some(MotorDirection::Forward)
                    };

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
