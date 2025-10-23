/*
Copyright 2025 Tyler Clarke

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/
// a nice abstraction for dealing with circuitdojo boards

use std::collections::HashMap;
use std::io::{Read, Write};
use std::slice::Iter;

use crate::error::Result;
use crate::{CircuitDojoError, connection::*};

use ringbuf::traits::Split;
use ringbuf::{CachingCons, CachingProd};
use ringbuf::{HeapRb, SharedRb};
use ringbuf::{consumer::Consumer, producer::Producer};
use std::sync::Arc;

#[derive(Copy, Clone)]
pub enum PinType {
    DigitalPullup,
    Digital,
    Analog,
}

#[derive(Copy, Clone)]
pub enum PinMode {
    Unset,
    Input,
    Output,
}

#[derive(Debug, Copy, Clone)]
pub enum PinStatus {
    NoStatus, // the pin is not configured for input or output
    DigitalOutputting(bool),
    DigitalInputting(bool),
    DigitalPullupInputting(bool),
    AnalogOutputting(u16),
    AnalogInputting(u16),
}

pub struct PinData {
    pub tp: PinType,
    pub mode: PinMode,
    pub hw_id: u8,
    pub ident: String,
    pub status: PinStatus, // not guaranteed to synchronize with
                           // PinMode or PinType
}

pub struct Board {
    pins: Vec<PinData>,
    board_name: String,
    min_sample: u16,
    mapped_pins_hwids: HashMap<u8, usize>,
    commands: CachingProd<Arc<HeapRb<Command>>>, // commands we're spraying to the connection
    // inside a worker thread
    events: CachingCons<Arc<HeapRb<BoardEvent>>>,
}

#[derive(Debug)]
enum BoardEvent {
    PinState(u8, PinStatus),
}

impl Board {
    pub fn new(port: impl AsRef<str>, baud: u32) -> Result<Self> {
        let mut conn = Connection::new(port.as_ref(), baud)?;
        conn.begin()?;
        conn.write_command(Command::RequestBoardParameters)?;
        let mut board_name = None;
        let mut min_sample = None;
        let mut pins = vec![];
        while board_name.is_none() || min_sample.is_none() {
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
                            status: PinStatus::NoStatus,
                        })
                    }
                    _ => {} // ignore all other events during setup mode
                }
            }
        }
        let mut mapped_pins_hwids = HashMap::new();
        for (i, pin) in pins.iter().enumerate() {
            mapped_pins_hwids.insert(pin.hw_id, i);
        }
        let (command_tx, command_rx) = HeapRb::new(32).split();
        let (event_tx, event_rx) = HeapRb::new(32).split();
        std::thread::spawn(Self::worker(command_rx, event_tx, conn));
        Ok(Self {
            min_sample: min_sample.unwrap(),
            board_name: board_name.unwrap(),
            mapped_pins_hwids,
            pins,
            commands: command_tx,
            events: event_rx,
        })
    }

    fn worker(
        mut commands: impl Consumer<Item = Command> + Send + 'static,
        mut events: impl Producer<Item = BoardEvent> + Send + 'static,
        mut connection: Connection,
    ) -> Box<dyn FnOnce() -> () + Send> {
        Box::new(move || {
            loop {
                let err = connection.wait_incoming();
                match err {
                    Ok(_) => {}
                    Err(CircuitDojoError::TimedOut) => {}
                    _ => {
                        err.unwrap();
                    }
                }
                for event in connection.events() {
                    match event {
                        Event::BoardError(command) => {
                            println!("failed to {:?}, synchronization issues may occur", command);
                        }
                        Event::DigitalPinStateChange(pin, state) => {
                            events
                                .try_push(BoardEvent::PinState(
                                    pin,
                                    PinStatus::DigitalInputting(state),
                                ))
                                .unwrap();
                        }
                        _ => {}
                    }
                }
                for command in commands.pop_iter() {
                    println!("running {:?}", command);
                    let _ = connection.write_command(command);
                }
            }
        })
    }

    pub fn get_name(&self) -> &str {
        &self.board_name
    }

    pub fn pins(&self) -> Iter<PinData> {
        self.pins.iter()
    }

    pub fn update(&mut self) -> Result<()> {
        // read incoming events and make changes
        for event in self.events.pop_iter() {
            match event {
                BoardEvent::PinState(pin, state) => {
                    let pindex = self
                        .mapped_pins_hwids
                        .get(&pin)
                        .ok_or(CircuitDojoError::InvalidPin(pin))?;
                    self.pins.get_mut(*pindex).unwrap().status = state;
                }
            }
        }
        Ok(())
    }

    pub fn set_output(&mut self, pin_num: u8) -> Result<()> {
        let pindex = self
            .mapped_pins_hwids
            .get(&pin_num)
            .ok_or(CircuitDojoError::InvalidPin(pin_num))?;
        let pin = self.pins.get_mut(*pindex).unwrap(); // unwrap is fine here: the index must be valid to have been returned from the mapping
        pin.mode = PinMode::Output;
        self.commands
            .try_push(Command::SetPinModeOutput(pin_num))
            .unwrap();
        Ok(())
    }

    pub fn set_input(&mut self, pin_num: u8) -> Result<()> {
        let pindex = self
            .mapped_pins_hwids
            .get(&pin_num)
            .ok_or(CircuitDojoError::InvalidPin(pin_num))?;
        let pin = self.pins.get_mut(*pindex).unwrap(); // unwrap is fine here: the index must be valid to have been returned from the mapping
        pin.mode = PinMode::Input;
        self.commands
            .try_push(Command::SetPinModeInput(pin_num))
            .unwrap();
        Ok(())
    }

    pub fn digital_write(&mut self, pin_num: u8, value: bool) -> Result<()> {
        let pindex = self
            .mapped_pins_hwids
            .get(&pin_num)
            .ok_or(CircuitDojoError::InvalidPin(pin_num))?;
        let pin = self.pins.get_mut(*pindex).unwrap(); // unwrap is fine here: the index must be valid to have been returned from the mapping
        pin.status = PinStatus::DigitalOutputting(value);
        if let PinMode::Output = pin.mode {
            self.commands
                .try_push(Command::SetDigitalPinValue(pin_num, value))
                .unwrap();
        } else {
            return Err(CircuitDojoError::InvalidPin(pin_num));
        }
        Ok(())
    }

    pub fn subscribe(&mut self, wavelength: u16) -> Result<()> {
        self.commands
            .try_push(Command::Subscribe(wavelength))
            .unwrap();
        Ok(())
    }
}
