use std::time::Duration;

use crate::AppState;
use crate::bus::event::{Event, Mode};

pub async fn run(app_state: AppState) {
    let mut bus_rx = app_state.bus.subscribe();
    let mut mode = Mode::Manual;

    let mut tick = tokio::time::interval(Duration::from_millis(200));

    loop {
        tokio::select! {
            Ok(event)=bus_rx.recv() => {
                if let Event::ModeCommand(new_mode) = event {
                    mode = new_mode.mode;
                    println!("mode changed to {:?}", mode);
                }
            }
            _ = tick.tick() => {
                if mode == Mode::Manual {
                    // TODO do something?
                    // let cmd = auto_wander(last_distance);
                    // bus_rx.publish(cmd);
                }
                if mode == Mode::Automatic {
                    // TODO do something?
                }
            }
        }
    }
}
