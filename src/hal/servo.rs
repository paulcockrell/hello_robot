use anyhow::Result;
use rppal::i2c::I2c;
use std::thread::sleep;
use std::time::Duration;

// PCA9685 Servo
pub struct Servo {
    i2c: I2c,
    addr: u16,
}

impl Servo {
    const MODE1: u8 = 0x00;
    const PRESCALE: u8 = 0xFE;
    const LED0_ON_L: u8 = 0x06;

    const OSC_CLOCK: f32 = 25_000_000.0;
    const PWM_RES: f32 = 4_096.0;

    pub fn new(addr: u16) -> Result<Servo> {
        let mut i2c = I2c::new()?;
        i2c.set_slave_address(addr)?;

        Ok(Servo { i2c, addr })
    }

    fn write(&mut self, reg: u8, val: u8) {
        self.i2c.smbus_write_byte(reg, val).unwrap();
    }

    fn read(&mut self, reg: u8) -> u8 {
        self.i2c.smbus_read_byte(reg).unwrap()
    }

    pub fn set_pwm_freq(&mut self, freq_hz: f32) {
        let prescale_val = (Self::OSC_CLOCK / (Self::PWM_RES * freq_hz)) - 1.0;
        let prescale = prescale_val.round() as u8;

        let old_mode = self.read(Self::MODE1);
        let sleep_mode = (old_mode & 0x7F) | 0x10; // sleep

        self.write(Self::MODE1, sleep_mode);
        self.write(Self::PRESCALE, prescale);
        self.write(Self::MODE1, old_mode);

        sleep(Duration::from_millis(5));

        self.write(Self::MODE1, old_mode | 0x80); // restart
    }

    pub fn set_pwm(&mut self, channel: u8, on: u16, off: u16) {
        let base: u8 = Self::LED0_ON_L + 4 * channel;

        self.write(base, (on & 0xFF) as u8);
        self.write(base + 1, (on >> 8) as u8);

        self.write(base + 2, (off & 0xFF) as u8);
        self.write(base + 3, (off >> 8) as u8);
    }

    pub fn set_angle(&mut self, channel: u8, percent: u8, min: u16, max: u16) {
        let pulse = map_range(percent as f32, 0.0, 100.0, min as f32, max as f32) as u16;
        self.set_pwm(channel, 0, pulse);
    }
}

fn map_range(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    out_min + (x - in_min) * (out_max - out_min) / (in_max - in_min)
}
