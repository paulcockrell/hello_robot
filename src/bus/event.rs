#[derive(Debug, Clone)]
pub enum MotorCommand {}

#[derive(Debug, Clone)]
pub enum MotorState {}

#[derive(Debug, Clone)]
pub enum LedCommand {}

#[derive(Debug, Clone)]
pub enum Event {
    MotorCommand(MotorCommand),
    MotorState(MotorState),
    UltrasoundReading(f64),
    CameraFrameReady,
    LedCommand(LedCommand),
    Shutdown,
}
