use crate::{
    bus::{
        event::{Event, Led, Ultrasound},
        event_bus::EventBus,
    },
    hal::neopixel::Neopixel,
};
use tokio::sync::mpsc;

pub async fn run(bus: EventBus) {
    let mut bus_rx = bus.subscribe();
    let bus_tx = bus.clone();

    let (tx, mut rx) = mpsc::channel::<Ultrasound>(16);

    let leds_task = tokio::task::spawn_blocking(move || {
        let mut neopixel = Neopixel::new().expect("Neopixel failed");
        let mut last_distance_i = 0_i32;

        while let Some(data) = rx.blocking_recv() {
            let distance_i = (data.distance * 2.0) as i32;

            if distance_i != last_distance_i {
                last_distance_i = distance_i;

                let (red, green, blue) = distance_to_rgb(data.distance);
                let brightness = calculate_brightness(red, green, blue);

                if let Err(e) = neopixel.set_pixels(red, green, blue, 0) {
                    eprintln!("Neopixel error: {e}");
                }

                bus_tx.publish(Event::Led(Led {
                    red,
                    green,
                    blue,
                    brightness: brightness.clamp(0.0, 255.0) as u8,
                }));
            }
        }
    });

    loop {
        match bus_rx.recv().await {
            Ok(Event::Ultrasound(cmd)) => {
                let _ = tx.send(cmd).await;
            }
            Ok(Event::Shutdown) => {
                println!("LEDs node shutting down");
                break;
            }
            Err(_) => break,
            _ => {}
        }
    }

    drop(tx);

    if let Err(e) = leds_task.await {
        eprintln!("LED task crashed: {e}");
    }
}

// Convert distance to a red-to-green scale for neopixels
fn distance_to_rgb(distance: f64) -> (u8, u8, u8) {
    let d = distance.clamp(0.0, 100.0);
    let t = d / 100.0;

    let red = (255.0 * (1.0 - t)) as u8;
    let green = (255.0 * t) as u8;
    let blue = 0u8;

    (red, green, blue)
}

fn calculate_brightness(red: u8, green: u8, blue: u8) -> f32 {
    let rf = red as f32;
    let gf = green as f32;
    let bf = blue as f32;

    0.2126 * rf + 0.7152 * gf + 0.0722 * bf
}
