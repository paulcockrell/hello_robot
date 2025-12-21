use crate::{AppState, bus::event::Event, hal::camera::Camera};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

pub async fn run(app_state: AppState) {
    let mut bus_rx = app_state.bus.subscribe();

    let running = Arc::new(AtomicBool::new(true));
    let running_thread = running.clone();

    let task = tokio::task::spawn_blocking(move || {
        let mut camera = Camera::new().expect("Could not setup camera");

        while running_thread.load(Ordering::Relaxed) {
            if let Ok(jpeg) = camera.frame_jpeg() {
                *app_state.camera.latest_frame.lock().unwrap() = jpeg;
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    });

    while let Ok(event) = bus_rx.recv().await {
        if matches!(event, Event::Shutdown) {
            println!("Camera node shutting down");
            break;
        }
    }

    running.store(false, Ordering::Relaxed);
    let _ = task.await;
}
