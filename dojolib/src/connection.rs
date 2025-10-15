use std::borrow::Cow;
use crate::opcodes::{miso, mosi};
use crate::CircuitDojoError;
use crate::Result;

pub struct Connection {
    port: Box<dyn serialport::SerialPort>,
}


#[derive(Debug)]
pub enum PinType {
    Digital,
    DigitalPullup,
    Analog,
}

#[derive(Debug)]
pub struct PinCapabilities {
    pub id: u8,
    pub pin_type: PinType,
    pub identifier: String,
}

#[derive(Debug)]
pub struct BoardCapabilities {
    pub pins: Vec<PinCapabilities>,
    pub name: String,
    pub min_sample_rate: u16,
}

impl Connection {
    pub fn new<'a>(port: impl Into<Cow<'a, str>>, baud: u32) -> serialport::Result<Self> {
        Ok(Self {
            port: serialport::new(port, baud)
                .dtr_on_open(false)
                .timeout(std::time::Duration::from_secs(1)) // after 1 seconds of not receiving data when data is expected, fail!
                .open()?,
        })
    }

    pub fn block_read_byte(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.port.read(&mut buf)?;
        Ok(buf[0])
    }

    pub fn block_read_nullt_string(&mut self) -> Result<String> {
        let mut strbuf = vec![];
        let mut i = 0;
        loop {
            let byte = self.block_read_byte()?;
            if byte == 0 {
                break;
            }
            strbuf.push(byte);
            i += 1;
        }
        Ok(String::from_utf8(strbuf).unwrap())
    }

    pub fn block_on_instruction(&mut self, instruction: &[u8]) -> Result<()> {
        self.port.write(instruction)?;
        let byte = self.block_read_byte()?;
        if byte == miso::ERROR {
            Err(CircuitDojoError::BoardError)
        } else if byte == miso::ACK {
            Ok(())
        } else {
            Err(CircuitDojoError::SynchronizationError(format!(
                "Expected DC_ACK, got {}",
                byte
            )))
        }
    }

    pub fn force_byte(&mut self, byte: u8) -> Result<()> {
        let result = self.block_read_byte()?;
        if result != byte {
            Err(CircuitDojoError::SynchronizationError(format!(
                "Expected {byte}, got {result}"
            )))
        } else {
            Ok(())
        }
    }

    pub fn begin(&mut self) -> Result<()> {
        let mut retry_limit = 5;
        loop {
            match self.block_on_instruction(&[mosi::PLEASE_ESTABLISH]) {
                Ok(()) => {
                    return Ok(());
                }
                Err(e) => {
                    if retry_limit == 0 {
                        return Err(e);
                    }
                }
            }
            retry_limit -= 1;
        }
    }

    pub fn request_capabilities(&mut self) -> Result<BoardCapabilities> {
        self.block_on_instruction(&[mosi::REQUEST_BOARD_PARAMETERS])?;
        self.force_byte(miso::SAMPLING_BOUNDS)?;
        let mut buf = [0; 2];
        self.port.read_exact(&mut buf)?;
        let sampling_minimum = u16::from_le_bytes(buf);
        let mut descriptors = vec![];
        loop {
            let byte = self.block_read_byte()?;
            if byte == miso::PIN_DESCRIPTION {
                let pin_descriptor = self.block_read_byte()?;
                let pin_number = pin_descriptor & 0b00111111;
                let is_analog = (pin_descriptor & 0x80) != 0;
                let is_pullup = (pin_descriptor & 0x40) != 0;
                let identifier = self.block_read_nullt_string()?;
                descriptors.push(PinCapabilities {
                    id: pin_number,
                    pin_type: if is_analog {
                        PinType::Analog
                    } else if is_pullup {
                        PinType::DigitalPullup
                    } else {
                        PinType::Digital
                    },
                    identifier,
                })
            } else if byte == miso::BOARD_DESCRIPTION {
                break;
            } else {
                return Err(CircuitDojoError::SynchronizationError(format!(
                    "Expected PIN_DESCRIPTION or BOARD_DESCRIPTION, got {}",
                    byte
                )));
            }
        }
        let board_description = self.block_read_nullt_string()?;
        Ok(BoardCapabilities {
            pins: descriptors,
            name: board_description,
            min_sample_rate: sampling_minimum,
        })
    }

    pub fn set_output(&mut self, pin: u8) -> Result<()> {
        self.block_on_instruction(&[mosi::SET_PIN_MODE_OUTPUT, pin])
    }

    pub fn set_input(&mut self, pin: u8) -> Result<()> {
        self.block_on_instruction(&[mosi::SET_PIN_MODE_INPUT, pin])
    }

    pub fn digital_write(&mut self, pin: u8, state: bool) -> Result<()> {
        self.port.write(&[pin | if state { 0x40 } else { 0x00 }])?;
        Ok(())
    }

    pub fn sample(&mut self, pin_count: usize) -> Result<Vec<(u8, bool)>> {
        self.block_on_instruction(&[mosi::RUN_ONE_SAMPLE])?;
        let mut output = vec![(0, false); pin_count];
        for i in 0..pin_count {
            let byte = self.block_read_byte()?;
            let index = byte & 0b00111111;
            output[i] = (index, byte & 0x40 > 0);
        }
        Ok(output)
    }
}
