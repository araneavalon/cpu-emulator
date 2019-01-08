
use std::rc::Rc;

use crate::bus::Bus;
use crate::connection::Connection;
use crate::error::Error;


#[derive(Debug, PartialEq, Eq, Hash)]
pub enum AddrByte {
  High,
  Low,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AddrBus {
  h: Rc<Bus>,
  l: Rc<Bus>,
}

impl AddrBus {
  pub fn new() -> AddrBus {
    AddrBus {
      h: Rc::new(Bus::new()),
      l: Rc::new(Bus::new()),
    }
  }

  pub fn h(&self) -> &Rc<Bus> {
    &self.h
  }

  pub fn l(&self) -> &Rc<Bus> {
    &self.l
  }

  pub fn connect(&self, connections: (Connection, Connection)) {
    self.h.connect(connections.0);
    self.l.connect(connections.1);
  }

  pub fn read(&self) -> Result<(u8, u8), Error> {
    Ok((self.h.read()?, self.l.read()?))
  }
}
