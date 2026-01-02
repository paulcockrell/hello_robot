use crate::{
    bus::{
        event::{Event, Ldr},
        event_bus::EventBus,
    },
    hal::ldr::LdrSensor,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

pub async fn run(bus: EventBus) {
    let mut bus_rx = bus.subscribe();
    let bus_tx = bus.clone();

    let running = Arc::new(AtomicBool::new(true));
    let running_thread = running.clone();

    let task = tokio::task::spawn_blocking(move || {
        let ldr = LdrSensor::new(19, 16, 20).unwrap();
        let mut last_reading: (u8, u8, u8) = (0, 0, 0);
        let mut tick: u32 = 0;

        while running_thread.load(Ordering::Relaxed) {
            tick = tick.wrapping_add(1);

            let readings = ldr.readings();

            if readings != last_reading || tick.is_multiple_of(10) {
                last_reading = readings;
                let (l_val, m_val, r_val) = readings;

                bus_tx.publish(Event::Ldr(Ldr {
                    l_val,
                    m_val,
                    r_val,
                }));
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    });

    while let Ok(event) = bus_rx.recv().await {
        if matches!(event, Event::Shutdown) {
            println!("LDR node shutting down");
            break;
        }
    }

    running.store(false, Ordering::Relaxed);
    let _ = task.await;
}
