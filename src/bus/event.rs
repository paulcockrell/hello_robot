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
pub struct Ultrasound {
    pub distance: f64,
}

#[derive(Debug, Clone)]
pub struct Ldr {
    pub l_val: u8,
    pub m_val: u8,
    pub r_val: u8,
}

#[derive(Debug, Clone)]
pub enum Event {
    MotorCommand(MotorCommand),
    ServoCommand(ServoCommand),
    Ultrasound(Ultrasound),
    Ldr(Ldr),
    Shutdown,
}
