// a nice abstraction for dealing with circuitdojo boards

use crate::connection::*;
use crate::error::Result;

pub enum PinType {
    DigitalPullup,
    Digital,
    Analog,
}

pub enum PinMode {
    Unset,
    Input,
    Output,
}

pub struct PinData {
    pub tp: PinType,
    pub mode: PinMode,
    pub hw_id: u8,
    pub ident: String,
}

pub struct Board {
    connection: Connection,
    pins: Vec<PinData>,
    board_name: String,
    min_sample: u16,
}

impl Board {
    pub fn new(port: impl AsRef<str>, baud: u32) -> Result<Self> {
        let mut conn = Connection::new(port.as_ref(), baud)?;
        conn.begin()?;
        conn.write_command(Command::RequestBoardParameters)?;
        let mut board_name = None;
        let mut min_sample = None;
        let mut pins = vec![];
        while board_name.is_none() && min_sample.is_none() {
            conn.wait_incoming()?;
            for event in conn.events() {
                match event {
                    Event::SamplingBounds(bounds) => {
                        min_sample = Some(bounds);
                    }
                    Event::BoardDescription(name) => {
                        board_name = Some(name);
                    }
                    Event::PinDescription(pin_id, is_analog, is_pullup, pin_name) => {
                        pins.push(PinData {
                            tp: if is_analog {
                                PinType::Analog
                            } else if is_pullup {
                                PinType::DigitalPullup
                            } else {
                                PinType::Digital
                            },
                            mode: PinMode::Unset,
                            hw_id: pin_id,
                            ident: pin_name,
                        })
                    }
                    _ => {} // ignore all other events during setup mode
                }
            }
        }
        Ok(Self {
            connection: conn,
            min_sample: min_sample.unwrap(),
            board_name: board_name.unwrap(),
            pins,
        })
    }
}
