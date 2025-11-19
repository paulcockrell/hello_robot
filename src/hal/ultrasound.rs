use anyhow::{Context, Result};
use rppal::gpio::{Gpio, InputPin, Level, OutputPin};
use std::time::Instant;
use tokio::time::Duration;

pub struct UltrasoundSensor {
    trig: OutputPin,
    echo: InputPin,
    max_wait: Duration,
    speed_of_sound: f32,
}

impl UltrasoundSensor {
    pub fn new(trig_pin: u8, echo_pin: u8) -> Result<UltrasoundSensor> {
        let gpio = Gpio::new().context("Failed to initialize GPIO")?;

        Ok(Self {
            trig: gpio.get(trig_pin)?.into_output_low(),
            echo: gpio.get(echo_pin)?.into_input_pulldown(),
            max_wait: Duration::from_micros(25_000), // 25ms = max sensor timeout
            speed_of_sound: 343.0,                   // m/s @ ~20 deg c
        })
    }

    pub fn measure_cm(&mut self) -> Option<u16> {
        // Trigger pulse
        self.trig.set_low();
        std::thread::sleep(Duration::from_micros(2));

        self.trig.set_high();
        std::thread::sleep(Duration::from_micros(10));
        self.trig.set_low();

        // Wait for echo to go high
        let start_wait = Instant::now();
        while self.echo.read() == Level::Low {
            if start_wait.elapsed() > self.max_wait {
                return None; // no wait
            }
        }

        // Echo High, start timing pulse
        let pulse_start = Instant::now();
        while self.echo.read() == Level::High {
            if pulse_start.elapsed() > self.max_wait {
                return None;
            }
        }

        let pulse_duration = pulse_start.elapsed();

        // Convert pulse time to distance
        let secs = pulse_duration.as_secs_f32();

        let distance_cm = (secs * self.speed_of_sound * 100.0) / 2.0;

        return Some(distance_cm as u16);
    }
}
