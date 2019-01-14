
use std::fmt;

use crate::math::*;
use crate::bus;
use crate::control;
use crate::error::Error;


#[derive(PartialEq, Eq)]
pub struct AddressRegister {
  control: control::AddressRegister,
  value: [u8; 2],
}

impl AddressRegister {
  pub fn new() -> AddressRegister {
    AddressRegister {
      control: control::AddressRegister::new(),
      value: [0x00, 0x00],
    }
  }
}

impl bus::Device<control::AddressRegister> for AddressRegister {
  fn update(&mut self, control: control::AddressRegister) -> Result<(), Error> {
    self.control = control;
    Ok(())
  }

  fn read(&self) -> Result<bus::State, Error> {
    Ok(bus::State {
      data: None,
      addr: if let control::Write::Write = self.control.Addr {
        Some(from_bytes(&self.value))
      } else {
        None
      },
    })
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    if let control::Read::Read = self.control.DataH {
      self.value[0] = state.read_data()?;
    }
    if let control::Read::Read = self.control.DataL {
      self.value[1] = state.read_data()?;
    }
    Ok(())
  }
}

impl fmt::Display for AddressRegister {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "AddressRegister({:#X})", from_bytes(&self.value))
  }
}

impl fmt::Debug for AddressRegister {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "AddressRegister({:#X} H={:?} L={:?} A={:?})", from_bytes(&self.value), self.control.DataH, self.control.DataL, self.control.Addr)
  }
}
