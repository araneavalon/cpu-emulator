
use std::fmt;
use crate::control::{
  Control,
  Address,
};
use crate::error::Result;
use super::BusComponent;


#[derive(Debug)]
pub struct AddressRegister {
  control: Control,
  value: u16,
}

impl AddressRegister {
  pub fn new() -> AddressRegister {
    AddressRegister {
      control: Control::new(),
      value: 0x0000,
    }
  }
}

impl fmt::Display for AddressRegister {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.control.address {
      Address::A => write!(f, "[ A]")?,
      _ =>          write!(f, "  A ")?,
    }
    if self.control.a.load {
      write!(f, " <=")?;
    } else {
      write!(f, " ==")?;
    }
    write!(f, " 0x{:04X}", self.value)?;
    Ok(())
  }
}

impl BusComponent for AddressRegister {
  fn name(&self) -> &'static str {
    "AddressRegister"
  }

  fn set_control(&mut self, control: Control) {
    self.control = control;
  }

  fn load(&mut self, value: u16) -> Result<()> {
    if self.control.a.load {
      self.value = value;
    }
    Ok(())
  }

  fn address(&self) -> Result<Option<u16>> {
    if let Address::A = self.control.address {
      Ok(Some(self.value))
    } else {
      Ok(None)
    }
  }
}
