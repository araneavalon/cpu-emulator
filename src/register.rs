
use std::fmt;

use crate::bus;
use crate::control;
use crate::error::Error;


#[derive(Debug, PartialEq, Eq)]
pub struct Register {
  control: control::Register,
  value: u8,
}

impl Register {
  pub fn new() -> Register {
    Register {
      control: control::Register::new(),
      value: 0x00,
    }
  }
}

impl bus::Device<control::Register> for Register {
  fn update(&mut self, control: control::Register) -> Result<(), Error> {
    self.control = control;
    Ok(())
  }

  fn read(&self) -> Result<bus::State, Error> {
    Ok(bus::State {
      data: if let control::ReadWrite::Write = self.control.Data {
        Some(self.value)
      } else {
        None
      },
      addr: None,
    })
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    if let control::ReadWrite::Read = self.control.Data {
      self.value = state.read_data()?;
    }
    Ok(())
  }
}

impl fmt::Display for Register {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "0x  {:02X} (Data={}) [Register]", self.value, self.control.Data)
  }
}
