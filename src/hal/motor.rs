use anyhow::{Context, Result};
use rppal::gpio::{Gpio, OutputPin};

pub struct Motor {
    in1: OutputPin,
    in2: OutputPin,
    en: OutputPin,
}

impl Motor {
    pub fn new(in1: u8, in2: u8, en: u8) -> Result<Motor> {
        let gpio = Gpio::new().context("Failed to initialize GPIO")?;
        let in1 = gpio.get(in1)?.into_output_low();
        let in2 = gpio.get(in2)?.into_output_low();
        let mut en = gpio.get(en)?.into_output();
        en.set_pwm_frequency(1000.0, 1.0)?;

        Ok(Self { in1, in2, en })
    }

    pub fn forward(&mut self, speed: u8 /* 0..100 */) -> Result<()> {
        self.in1.set_high();
        self.in2.set_low();
        let duty_cycle = speed as f64 / 100.0;
        Ok(self.en.set_pwm_frequency(1000.0, duty_cycle)?)
    }

    pub fn backward(&mut self, speed: u8 /* 0..100 */) -> Result<()> {
        self.in1.set_low();
        self.in2.set_high();
        let duty_cycle = speed as f64 / 100.0;
        Ok(self.en.set_pwm_frequency(1000.0, duty_cycle)?)
    }
}
