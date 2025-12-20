use anyhow::{Context, Result};
use rppal::gpio::{Gpio, InputPin};

pub struct LdrSensor {
    l_pin: InputPin, // 19
    m_pin: InputPin, // 16
    r_pin: InputPin, // 20
}

impl LdrSensor {
    pub fn new(l_pin_num: u8, m_pin_num: u8, r_pin_num: u8) -> Result<LdrSensor> {
        let gpio = Gpio::new().context("Failed to initialize GPIO")?;
        let l_pin = gpio
            .get(l_pin_num)
            .context("Failed to obtain left LDR pin")?
            .into_input();
        let m_pin = gpio
            .get(m_pin_num)
            .context("Failed to obtain middle LDR pin")?
            .into_input();
        let r_pin = gpio
            .get(r_pin_num)
            .context("Failed to obtain right LDR pin")?
            .into_input();

        Ok(Self {
            l_pin,
            m_pin,
            r_pin,
        })
    }

    pub fn readings(&self) -> (u8, u8, u8) {
        let l_pin_level = self.l_pin.read() as u8;
        let m_pin_level = self.m_pin.read() as u8;
        let r_pin_level = self.r_pin.read() as u8;

        (l_pin_level, m_pin_level, r_pin_level)
    }
}
