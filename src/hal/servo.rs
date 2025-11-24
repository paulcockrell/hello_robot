use pca9685_rppal::*;

pub struct Servo {
    pca: Pca9685,
}

impl Servo {
    pub fn new() -> anyhow::Result<Self> {
        let mut pca = Pca9685::new().expect("new Pca9685");
        pca.init().expect("Initialise PCA9685");
        pca.set_pwm_freq(50.0).expect("Set PCA9685 freq to 50hz"); // 50Hz, exactly like Clojure

        Ok(Self { pca })
    }

    pub fn set_angle(&mut self, percent: u8) -> anyhow::Result<()> {
        let angle = map_range(percent as i32, 0, 100, 300, 150) as u16;
        self.pca.set_pwm(0, 0, angle).expect("Set angle to {angle}");

        Ok(())
    }
}

fn map_range(value: i32, in_min: i32, in_max: i32, out_min: i32, out_max: i32) -> i32 {
    out_min + (value - in_min) * (out_max - out_min) / (in_max - in_min)
}
