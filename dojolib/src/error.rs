/*
Copyright 2025 Tyler Clarke

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

*/
#[derive(Debug)]
pub enum CircuitDojoError {
    BoardError,                   // the board sent 0xFE
    SynchronizationError(String), // unexpected unprocessable bytes were received
    IoError(std::io::Error),
    SerialportError(serialport::Error),
    TimedOut, // io timeout
    InvalidPin(u8), // tried to access a pin that does not exist
              // or cannot be accessed
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
