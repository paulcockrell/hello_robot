use serde::Serialize;

#[derive(Debug, Clone)]
pub enum MotorDirection {
    Forward,
    Backward,
    Left,
    Right,
    Stop,
}

#[derive(Debug, Clone)]
pub struct MotorCommand {
    pub direction: MotorDirection,
    pub speed: u8,
}

#[derive(Debug, Clone)]
pub struct ServoCommand {
    pub angle: u8,
}

#[derive(Debug, Clone)]
pub struct LedCommand {
    pub brightness: i8,
}

#[derive(Debug, Serialize, Clone)]
pub struct Ultrasound {
    pub distance: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct Ldr {
    pub l_val: u8,
    pub m_val: u8,
    pub r_val: u8,
}

#[derive(Debug, Serialize, Clone)]
pub struct Led {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub brightness: u8,
}

#[derive(Debug, Clone)]
pub enum Event {
    MotorCommand(MotorCommand),
    ServoCommand(ServoCommand),
    LedCommand(LedCommand),
    Ultrasound(Ultrasound),
    Ldr(Ldr),
    #[allow(dead_code)]
    Led(Led),
    Shutdown,
}
