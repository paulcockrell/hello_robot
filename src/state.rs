use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct RobotState {
    pub ldr_left: u8,
    pub ldr_middle: u8,
    pub ldr_right: u8,
    pub ultrasound: f64,
    pub neopixel_r: u8,
    pub neopixel_g: u8,
    pub neopixel_b: u8,
}
