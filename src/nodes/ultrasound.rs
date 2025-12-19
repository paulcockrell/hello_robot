use crate::{
    bus::{
        event::{Event, Ultrasound},
        event_bus::EventBus,
    },
    hal::ultrasound::UltrasoundSensor,
};
use std::time::Duration;

pub async fn run(bus: EventBus) {
    let mut rx = bus.subscribe();
    let mut us = UltrasoundSensor::new(11, 8).unwrap();
    let mut last_avg = 0.0;
    let mut avg = 0.0;

    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                let dist = us.measure_cm().unwrap_or(0);
                let new_value = dist.clamp(0, 100);
                avg = avg * 0.5 + (new_value as f64) * 0.3;

                if avg != last_avg {
                    last_avg = avg;

                    let data = Ultrasound{
                        distance: avg
                    };

                    bus.publish(Event::Ultrasound(data));
                }
            }
            msg = rx.recv() => {
                match msg {
                    Ok(Event::Shutdown) => {
                        println!("Ultrasound node shutting down");
                        break;
                    }
                    Ok(_) => {
                        // ignore other events
                    }
                    Err(_) => {
                        // channel closed → yield or exit
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        }
    }
}
