
use crate::error::Error;
use crate::control;


#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Addr {
  Full(u16),
  High(u8),
  Low(u8),
}

#[derive(Debug, PartialEq, Eq)]
pub struct State {
  pub data: Option<u8>,
  pub addr: Option<Addr>,
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
      Some(Addr::Full(value)) => Ok(value),
      Some(Addr::High(_)) => Err(Error::InvalidRead(String::from("addr:L"))),
      Some(Addr::Low(_)) => Err(Error::InvalidRead(String::from("addr:H"))),
      None => Err(Error::InvalidRead(String::from("addr:HL")))
    }
  }
}

pub trait Device<T: control::Trait> {
  fn update(&mut self, control: T) -> Result<(), Error>;
  fn read(&self) -> State;
  fn clk(&mut self, state: &State) -> Result<(), Error>;
}
