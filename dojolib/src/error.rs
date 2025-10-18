#[derive(Debug)]
pub enum CircuitDojoError {
    BoardError,                   // the board sent 0xFE
    SynchronizationError(String), // unexpected unprocessable bytes were received
    IoError(std::io::Error),
    SerialportError(serialport::Error),
    TimedOut, // io timeout
}

impl From<std::io::Error> for CircuitDojoError {
    fn from(error: std::io::Error) -> CircuitDojoError {
        if let std::io::ErrorKind::TimedOut = error.kind() {
            CircuitDojoError::TimedOut
        } else {
            CircuitDojoError::IoError(error)
        }
    }
}

impl From<serialport::Error> for CircuitDojoError {
    fn from(error: serialport::Error) -> CircuitDojoError {
        CircuitDojoError::SerialportError(error)
    }
}

pub type Result<T> = std::result::Result<T, CircuitDojoError>;
