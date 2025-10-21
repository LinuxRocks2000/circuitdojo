// a low-level event queue manager for circuitdojo boards
// stores a queue of commands waiting for ack/error and a queue of events returned from the board
// when you send a command to the board, it's immediately serialized and written to the serial buffer, and
// also pushed to the front of the waiting deque
// then the wait_incoming function processes incoming ack/error/data relating to the top of the queue, pops
// off the top, and optionally pushes an event to the event queue.
//
// this structure allows you to send several commands asynchronously and let the board (very slow compared to the computer!) chew on them for a bit
// without delaying the application's serial thread. it also allows the board to send asynchronous events we've subscribed to
// (for instance, pin state updates) without corrupting some other command.
//
// blocking commands should just call wait_incoming a bunch until an Event they care about is pushed to queue.
//
// note that, being "low level", this structure doesn't track state or validate anything. it's
// just a serialization helper and an event-queue manager. it's highly recommended that you never manually
// use this and instead work with something sane like Board.

use crate::CircuitDojoError;
use crate::Result;
use crate::board::PinStatus;
use crate::opcodes::{miso, mosi};
use std::borrow::Cow;
use std::collections::VecDeque;
use std::collections::vec_deque::Drain;

#[derive(Debug)]
pub enum Command {
    // an outgoing command.
    PleaseEstablish,
    RequestBoardParameters,
    SetPinModeInput(u8),
    SetPinModeOutput(u8),
    RunOneSample,
    SetDigitalPinValue(u8, bool),
    Subscribe(u16),
}

#[derive(Debug)]
pub enum Event {
    // an event from the board.
    BoardError(Command), // the board returned Error for some command; we probably want to log it, and possibly warn the user!
    DigitalPinStateChange(u8, bool), // a digital pin's state changed
    AnalogPinStateChange(u8, u16), // an analog pin's state changed
    SamplingBounds(u16), // minimum time between samples (unenforced)
    PinDescription(u8, bool, bool, String), // description of a pin
    // pin_id, analog, pullup support, pin identifier
    BoardDescription(String), // description of this board
                              // just board_name right now
}

pub struct Connection {
    port: Box<dyn serialport::SerialPort>,
    waiting_commands: VecDeque<Command>,
    events: VecDeque<Event>,
}

impl Connection {
    pub fn new<'a>(port: impl Into<Cow<'a, str>>, baud: u32) -> serialport::Result<Self> {
        Ok(Self {
            port: serialport::new(port, baud)
                .dtr_on_open(false)
                .timeout(std::time::Duration::from_secs(1)) // after 1s of not receiving data when data is expected, fail!
                .open()?,
            waiting_commands: VecDeque::new(),
            events: VecDeque::new(),
        })
    }

    pub fn block_read_byte(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.port.read(&mut buf)?;
        Ok(buf[0])
    }

    pub fn write_byte(&mut self, byte: u8) -> Result<()> {
        let buf = [byte];
        self.port.write(&buf)?;
        Ok(())
    }

    pub fn block_read_nullt_string(&mut self) -> Result<String> {
        let mut strbuf = vec![];
        loop {
            let byte = self.block_read_byte()?;
            if byte == 0 {
                break;
            }
            strbuf.push(byte);
        }
        Ok(String::from_utf8(strbuf).unwrap())
    }

    pub fn write_command(&mut self, command: Command) -> Result<()> {
        match command {
            Command::PleaseEstablish => {
                self.write_byte(mosi::PLEASE_ESTABLISH)?;
            }
            Command::RequestBoardParameters => {
                self.write_byte(mosi::REQUEST_BOARD_PARAMETERS)?;
            }
            Command::RunOneSample => {
                self.write_byte(mosi::RUN_ONE_SAMPLE)?;
            }
            Command::SetDigitalPinValue(pin, value) => {
                self.write_byte(pin | if value { 0x40 } else { 0x00 })?;
            }
            Command::SetPinModeInput(pin) => {
                self.write_byte(mosi::SET_PIN_MODE_INPUT)?;
                self.write_byte(pin)?;
            }
            Command::SetPinModeOutput(pin) => {
                self.write_byte(mosi::SET_PIN_MODE_OUTPUT)?;
                self.write_byte(pin)?;
            }
            Command::Subscribe(wavelength) => {
                self.write_byte(mosi::SUBSCRIBE)?;
                self.port.write(&wavelength.to_le_bytes())?;
            }
        }
        self.waiting_commands.push_back(command);
        Ok(())
    }

    pub fn wait_incoming(&mut self) -> Result<()> {
        let byte = self.block_read_byte()?;
        if byte < 128 {
            // this is a digital pin value set
            self.events
                .push_back(Event::DigitalPinStateChange(byte & 0x3F, byte & 0x40 > 0));
            return Ok(());
        }
        match byte {
            miso::ERROR => {
                // a board error does not necessarily terminate;
                // we need to log this to the queue and proceed.
                if let Some(command) = self.waiting_commands.pop_front() {
                    self.events.push_back(Event::BoardError(command));
                } else {
                    return Err(CircuitDojoError::SynchronizationError(format!(
                        "Unexpected ERROR: There is no command in queue. Possible board malfunction."
                    )));
                }
            }
            miso::ACK => {
                // this is uninteresting. just discard.
                if let Some(Command::PleaseEstablish) = self.waiting_commands.pop_front() {
                    self.waiting_commands.retain(|m| {
                        // if the board ACKs a PleaseEstablish, there's probably another PleaseEstablish
                        // in the buffer that never got acked (because it was sent before the board booted)
                        if let Command::PleaseEstablish = m {
                            false
                        } else {
                            true
                        }
                    })
                }
            }
            miso::SAMPLING_BOUNDS => {
                let mut buf = [0; 2];
                self.port.read_exact(&mut buf)?;
                self.events
                    .push_back(Event::SamplingBounds(u16::from_le_bytes(buf)));
            }
            miso::PIN_DESCRIPTION => {
                let pin_descriptor = self.block_read_byte()?;
                let pin_name = self.block_read_nullt_string()?;
                self.events.push_back(Event::PinDescription(
                    pin_descriptor & 0x3F,
                    (pin_descriptor & 0x80) != 0,
                    (pin_descriptor & 0x40) != 0,
                    pin_name,
                ));
            }
            miso::BOARD_DESCRIPTION => {
                let board_name = self.block_read_nullt_string()?;
                self.events.push_back(Event::BoardDescription(board_name));
            }
            _ => {
                return Err(CircuitDojoError::SynchronizationError(format!(
                    "Expected control byte, got {}",
                    byte
                )));
            }
        }
        Ok(())
    }

    pub fn yoink_event(&mut self, selector: impl Fn(&Event) -> bool) -> Option<Event> {
        // if there is an event in the queue that matches selector,
        // swap remove it from the queue and return it.
        let mut pick = None;
        for (i, event) in self.events.iter().enumerate() {
            if selector(event) {
                pick = Some(i);
                break;
            }
        }
        if let Some(pick) = pick {
            self.events.swap_remove_back(pick)
        } else {
            None
        }
    }

    pub fn event_wait(&mut self, selector: impl Fn(&Event) -> bool + Copy) -> Result<Event> {
        // call wait_incoming repeatedly until it yields an event
        loop {
            self.wait_incoming()?;
            if let Some(evt) = self.yoink_event(selector) {
                return Ok(evt);
            }
        }
    }

    pub fn begin(&mut self) -> Result<()> {
        let mut retry_limit = 5;
        while retry_limit > 0 {
            self.write_command(Command::PleaseEstablish)?;
            if let Ok(()) = self.wait_incoming() {
                self.port
                    .set_timeout(std::time::Duration::from_millis(10))?;
                return Ok(());
            }
            retry_limit -= 1;
        }
        Err(CircuitDojoError::TimedOut)
    }

    pub fn events(&mut self) -> Drain<Event> {
        self.events.drain(..)
    }
}
