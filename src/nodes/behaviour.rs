use std::time::Duration;

use rand::seq::IndexedRandom;

use crate::AppState;
use crate::bus::event::{DriveIntent, Event, Mode};

pub async fn run(app_state: AppState) {
    let mut bus_rx = app_state.bus.subscribe();
    let bus_tx = app_state.bus.clone();

    let mut mode = Mode::Manual;
    let mut last_distance = 999.9;
    let mut tick = tokio::time::interval(Duration::from_millis(200));
    let mut new_intent: Option<DriveIntent> = None;
    let mut last_intent: Option<DriveIntent> = None;

    loop {
        tokio::select! {
            Ok(event)=bus_rx.recv() => {
                match event {
                    Event::ModeCommand(new_mode) =>  {
                        mode = new_mode.mode;
                        println!("mode changed to {:?}", mode);
                    },
                    Event::Ultrasound(ultrasound) => last_distance = ultrasound.distance,
                    _ => {}
                }
            }
            _ = tick.tick() => {
                if mode == Mode::Manual {
                    // Do nothing, the user has command
                }
                if mode == Mode::Automatic {
                    if last_distance < 10.0 {
                        if last_intent.as_ref() == Some(&DriveIntent::Forward) {
                            new_intent = Some(random_avoidance_intent());
                        } else {
                            new_intent = last_intent.clone();
                        }
                    } else {
                        new_intent = Some(DriveIntent::Forward);
                    }

                    if new_intent.as_ref() != last_intent.as_ref() {
                        if let Some(intent) = new_intent.as_ref() {
                            bus_tx.publish(Event::DriveIntent(intent.clone()));
                            println!("[AUTO] New intent selected {:?}", intent);
                        }

                        last_intent = new_intent.clone();
                    }
                }
            }
        }
    }
}

fn random_avoidance_intent() -> DriveIntent {
    let intents = [
        DriveIntent::TurnLeft {},
        DriveIntent::TurnRight {},
        DriveIntent::Backward {},
    ];
    let mut rng = rand::rng();

    intents.choose(&mut rng).unwrap().clone()
}
