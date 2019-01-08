
use std::cell::RefCell;

use crate::error::Error;
use crate::connection::Connection;


#[derive(Debug, PartialEq, Eq)]
pub struct Bus {
  connections: RefCell<Vec<Connection>>,
}

impl Bus {
  pub fn new() -> Bus {
    Bus {
      connections: RefCell::new(Vec::new()),
    }
  }

  pub fn connect(&self, connection: Connection) {
    self.connections.borrow_mut().push(connection);
  }

  pub fn read(&self) -> Result<u8, Error> {
    let results: Result<Vec<Option<u8>>, Error> = self.connections.borrow().iter().map(|c| c.read()).collect();
    let values: Vec<u8> = results?.iter().cloned().filter_map(|c| c).collect();
    if values.len() != 1 {
      Err(Error::AmbiguousBus(values.len()))
    } else {
      Ok(values[0])
    }
  }
}

