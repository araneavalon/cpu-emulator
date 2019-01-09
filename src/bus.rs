
use crate::error::Error;
use crate::control;


#[derive(Debug, PartialEq, Eq)]
pub struct State {
  pub data: Option<u8>,
  pub addr: Option<u16>,
}

impl State {
  pub fn read_data(&self) -> Result<u8, Error> {
    match self.data {
      Some(value) => Ok(value),
      None => Err(Error::InvalidRead(String::from("data"))),
    }
  }

  pub fn read_addr(&self) -> Result<u16, Error> {
    match self.addr {
      Some(value) => Ok(value),
      None => Err(Error::InvalidRead(String::from("addr"))),
    }
  }

  pub fn data(&self) -> Option<u8> {
    self.data
  }

  pub fn addr(&self) -> Option<u16> {
    self.addr
  }
}

pub trait Device<T: control::Trait> {
  fn update(&mut self, control: T) -> Result<(), Error>;
  fn read(&self) -> State;
  fn clk(&mut self, state: &State) -> Result<(), Error>;
}
