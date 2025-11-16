use rppal::gpio::Gpio;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use tokio::task;
use tokio::time::{Duration, sleep};

#[derive(Debug, Deserialize)]
struct RobotState {
    ldr_left_value: u8,
    ldr_middle_value: u8,
    ldr_right_value: u8,
    last_cmd: String,
}

fn ldr(state: Arc<Mutex<RobotState>>) {
    let l_pin = Gpio::new().unwrap().get(19).unwrap().into_input();
    let m_pin = Gpio::new().unwrap().get(16).unwrap().into_input();
    let r_pin = Gpio::new().unwrap().get(20).unwrap().into_input();

    loop {
        let l_pin_level = l_pin.read();
        let m_pin_level = m_pin.read();
        let r_pin_level = r_pin.read();

        {
            let mut s = state.lock().unwrap();
            s.ldr_left_value = l_pin_level as u8;
            s.ldr_middle_value = m_pin_level as u8;
            s.ldr_right_value = r_pin_level as u8;
        }

        println!(
            "LDR left: {:?}, middle: {:?}, right: {:?}",
            l_pin_level, m_pin_level, r_pin_level
        );

        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

async fn socket_responder(sock_path: &str, state: Arc<Mutex<RobotState>>) -> anyhow::Result<()> {
    let _ = std::fs::remove_file(sock_path);

    let listener = UnixListener::bind(sock_path)?;
    println!("Listening on Unix socket {}", sock_path);

    loop {
        let (mut stream, _) = listener.accept().await?;

        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await?;

        let msg = String::from_utf8_lossy(&buf).to_string();

        {
            let mut s = state.lock().unwrap();
            s.last_cmd = msg.clone();
        }

        println!("CMD = {}", msg);
    }
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(RobotState {
        ldr_left_value: 0,
        ldr_middle_value: 0,
        ldr_right_value: 0,
        last_cmd: "none".into(),
    }));

    //
    // Task A - LDR polling
    //
    {
        let state = Arc::clone(&state);
        task::spawn_blocking(move || ldr(state));
    }

    //
    // Task B - Unix Socket responder
    // Send comands example: echo '{"cmd":"hello"}' | socat - UNIX-CONNECT:/tmp/robot.sock
    //
    {
        let state = Arc::clone(&state);
        let sock_path = "/tmp/robot.sock";
        task::spawn(async move {
            socket_responder(sock_path, state).await.unwrap();
        });
    }

    //
    // Task C - Global robot logic loop
    //
    loop {
        let s = state.lock().unwrap();
        println!("ROBOT STATE: {:?}", *s);
        drop(s);

        sleep(Duration::from_secs(1)).await;
    }
}
