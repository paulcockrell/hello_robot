use std::time::Duration;

use crate::{
    bus::{bus::EventBus, event::Event},
    hal::ldr::Ldr,
};

pub async fn run(bus: EventBus) {
    let mut rx = bus.subscribe();
    let ldr = Ldr::new(19, 16, 20).unwrap();
    let mut last_reading: (u8, u8, u8) = (0, 0, 0);

    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(200))=>{
                let readings = ldr.readings();

                if readings != last_reading {
                    last_reading = readings;
                    let (l_val, m_val, r_val) = readings;
                    println!("l_val={l_val}, m_val={m_val}, r_val={r_val}");
                }

            }
            msg = rx.recv() => {
                if matches!(msg, Ok(Event::Shutdown)) {
                    println!("LDR node shutting down");
                    break;
                }
            }
        }
    }
}
