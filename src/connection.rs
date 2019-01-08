
use std::rc::Rc;
use std::fmt;

use crate::error::Error;

use crate::register;
use crate::mux;
use crate::input;
use crate::program_counter;
use crate::ram;


#[derive(PartialEq, Eq)]
pub enum Connection {
  Register(Rc<register::Register>, register::Control),
  Mux(Rc<mux::Mux>),
  Input(Rc<input::Input>),
  ProgramCounter(Rc<program_counter::ProgramCounter>, program_counter::Control),
  Ram(Rc<ram::Ram>),
}

impl Connection {
  pub fn read(&self) -> Result<Option<u8>, Error> {
    match self {
      Connection::Register(s, c) => s.read(c),
      Connection::Mux(s) => s.read(),
      Connection::Input(s) => s.read(),
      Connection::ProgramCounter(s, c) => s.read(c),
      Connection::Ram(s) => s.read(),
    }
  }
}

impl fmt::Debug for Connection {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Connection::Register(_, _) => write!(f, "Connection::Register"),
      Connection::Mux(_) => write!(f, "Connection::Mux"),
      Connection::Input(_) => write!(f, "Connection::Input"),
      Connection::ProgramCounter(_, _) => write!(f, "Connection::ProgramCounter"),
      Connection::Ram(_) => write!(f, "Connection::Ram"),
    }
  }
}
