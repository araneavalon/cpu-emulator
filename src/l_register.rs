
use std::fmt;

use crate::bus;
use crate::control;
use crate::error::Error;


#[derive(PartialEq, Eq)]
pub struct LRegister {
  control: control::LRegister,
  value: u8
}

impl LRegister {
  pub fn new() -> LRegister {
    LRegister {
      control: control::LRegister::new(),
      value: 0x00,
    }
  }
}

impl bus::Device<control::LRegister> for LRegister {
  fn update(&mut self, control: control::LRegister) -> Result<(), Error> {
    self.control = control;
    Ok(())
  }

  fn read(&self) -> bus::State {
    bus::State {
      data: if let control::ReadWrite::Write = self.control.Data {
        Some(self.value)
      } else {
        None
      },
      addr: if let control::Write::Write = self.control.Addr {
        Some(bus::Addr::High(self.value))
      } else {
        None
      },
    }
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    if let control::ReadWrite::Read = self.control.Data {
      self.value = state.read_data()?;
    }
    Ok(())
  }
}

impl fmt::Display for LRegister {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "LRegister({:#X})", self.value)
  }
}

impl fmt::Debug for LRegister {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "LRegister({:#X} D={:?} A={:?})", self.value, self.control.Data, self.control.Addr)
  }
}
