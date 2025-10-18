/*
    This is the library that drives communication to the CircuitDojo board.
    It's two-layered: you have the very simple Connection structure, which has a bunch of functions
    for querying and controlling the board, and you have the nicer Board structure, which manages state,
    validates commands, and provides a nice pin interface.
*/
pub const DOJOLIB_VERSION: u8 = 1;

pub mod connection;
mod opcodes;
pub use connection::Connection; // allow raw connections
pub mod error;
pub use error::{CircuitDojoError, Result};
pub mod board;
pub use board::Board;

pub fn ports() -> Result<Vec<String>> {
    Ok(serialport::available_ports()?
        .into_iter()
        .map(|m| m.port_name)
        .collect())
}
