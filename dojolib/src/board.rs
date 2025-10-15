// a nice abstraction for dealing with circuitdojo boards

use crate::connection::Connection;
use crate::errors::Result;


pub struct Board {
    connection : Connection
}


impl Board {
    pub fn new(port : impl AsRef<str>) -> Result<Self> {
        let mut conn = Connection::new(port.as_ref())?;
        
    }
}