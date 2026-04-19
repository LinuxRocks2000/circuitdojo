/*
Copyright 2025 Tyler Clarke

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/
// a nice abstraction for dealing with circuitdojo boards

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::Result;
use crate::{CircuitDojoError, connection::*};

use crossbeam::channel::{Receiver, Sender, unbounded};

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
    AnalogOutputting(u8), // 8-bit "DAC" (simulated with PWM)
    AnalogInputting(u16), // 10-bit ADC
}

pub struct PinData {
    pub tp: PinType,
    pub mode: PinMode,
    pub hw_id: u8,
    pub ident: String,
    pub status: PinStatus, // not guaranteed to synchronize with
                           // PinMode or PinType
}

pub struct PinControls {
    pin: Rc<RefCell<PinData>>, // MUST be a mutable reference to a pin inside the board's vector of pins
    commands: Sender<Command>,
}

impl PinControls {
    pub fn digital_write(&mut self, value: bool) -> Result<()> {
        self.pin.borrow_mut().status = PinStatus::DigitalOutputting(value);
        if let PinMode::Output = self.pin.borrow().mode {
            self.commands
                .send(Command::SetDigitalPinValue(self.pin.borrow().hw_id, value))
                .unwrap();
        } else {
            return Err(CircuitDojoError::InvalidPin(self.pin.borrow().hw_id));
        }
        Ok(())
    }

    pub fn digital_read(&mut self) -> Result<bool> {
        match self.pin.borrow().status {
            PinStatus::DigitalInputting(value) => Ok(value),
            _ => Err(CircuitDojoError::InvalidPin(self.pin.borrow().hw_id)),
        }
    }

    pub fn set_output(&mut self) -> Result<()> {
        self.pin.borrow_mut().mode = PinMode::Output;
        self.pin.borrow_mut().status = PinStatus::DigitalOutputting(false);
        self.commands
            .send(Command::SetPinModeOutput(self.pin.borrow().hw_id))
            .unwrap();
        Ok(())
    }

    pub fn set_input(&mut self) -> Result<()> {
        self.pin.borrow_mut().mode = PinMode::Input;
        self.commands
            .send(Command::SetPinModeInput(self.pin.borrow().hw_id))
            .unwrap();
        Ok(())
    }

    pub fn analog_write(&mut self, value: u8) -> Result<()> {
        self.pin.borrow_mut().mode = PinMode::Output;
        self.pin.borrow_mut().status = PinStatus::AnalogOutputting(value);
        self.commands
            .send(Command::SetAnalogPinValue(self.pin.borrow().hw_id, value))
            .unwrap();
        Ok(())
    }

    pub fn disable(&mut self) -> Result<()> {
        self.pin.borrow_mut().mode = PinMode::Unset;
        self.pin.borrow_mut().status = PinStatus::NoStatus;
        self.commands
            .send(Command::Disable(self.pin.borrow().hw_id))
            .unwrap();
        Ok(())
    }

    pub fn hw_id(&self) -> u8 {
        self.pin.borrow().hw_id
    }

    pub fn status(&self) -> PinStatus {
        self.pin.borrow().status
    }

    pub fn tp(&self) -> PinType {
        self.pin.borrow().tp
    }

    pub fn mode(&self) -> PinMode {
        self.pin.borrow().mode
    }

    pub fn ident(&self) -> String {
        self.pin.borrow().ident.clone() // TODO: don't clone here
    }
}

pub struct PinIterator<'board> {
    board: &'board mut Board,
    pin_index: usize,
}

impl<'board> Iterator for PinIterator<'board> {
    type Item = PinControls;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.pin_index;
        self.pin_index += 1;
        if index >= self.board.pins.len() {
            return None;
        }
        Some(PinControls {
            pin: self.board.pins[index].clone(),
            commands: self.board.commands.clone(),
        })
    }
}

pub struct Board {
    pins: Vec<Rc<RefCell<PinData>>>,
    board_name: String,
    mapped_pins_hwids: HashMap<u8, usize>,
    commands: Sender<Command>, // commands we're spraying to the connection
    // inside a worker thread
    events: Receiver<BoardEvent>,
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
                        pins.push(Rc::new(RefCell::new(PinData {
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
                        })))
                    }
                    _ => {} // ignore all other events during setup mode
                }
            }
        }
        let mut mapped_pins_hwids = HashMap::new();
        for (i, pin) in pins.iter().enumerate() {
            mapped_pins_hwids.insert(pin.borrow().hw_id, i);
        }
        let (command_tx, command_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        std::thread::spawn(Self::worker(command_rx, event_tx, conn));
        Ok(Self {
            board_name: board_name.unwrap(),
            mapped_pins_hwids,
            pins,
            commands: command_tx,
            events: event_rx,
        })
    }

    fn worker(
        commands: Receiver<Command>,
        events: Sender<BoardEvent>,
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
                                .send(BoardEvent::PinState(
                                    pin,
                                    PinStatus::DigitalInputting(state),
                                ))
                                .unwrap();
                        }
                        Event::AnalogPinStateChange(pin, value) => {
                            events
                                .send(BoardEvent::PinState(pin, PinStatus::AnalogInputting(value)))
                                .unwrap();
                        }
                        _ => {}
                    }
                }
                for command in commands.try_iter() {
                    let _ = connection.write_command(command);
                }
            }
        })
    }

    pub fn get_name(&self) -> &str {
        &self.board_name
    }

    pub fn pins<'a>(&'a mut self) -> PinIterator<'a> {
        PinIterator {
            board: self,
            pin_index: 0,
        }
    }

    pub fn get_pin<'a>(&'a mut self, pin: u8) -> Option<PinControls> {
        let pin = self.mapped_pins_hwids.get(&pin)?;
        let pin = { self.pins.get(*pin)?.clone() };
        Some(PinControls {
            commands: self.commands.clone(),
            pin,
        })
    }

    pub fn update(&mut self) -> Result<()> {
        // read incoming events and make changes
        for event in self.events.try_iter() {
            match event {
                BoardEvent::PinState(pin, state) => {
                    let pindex = self
                        .mapped_pins_hwids
                        .get(&pin)
                        .ok_or(CircuitDojoError::InvalidPin(pin))?;
                    self.pins[*pindex].borrow_mut().status = state;
                }
            }
        }
        Ok(())
    }

    pub fn subscribe(&mut self, wavelength: u16) -> Result<()> {
        self.commands.send(Command::Subscribe(wavelength)).unwrap();
        Ok(())
    }
}
