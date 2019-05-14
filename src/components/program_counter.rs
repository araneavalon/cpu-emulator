
use std::fmt;
use crate::control::{
  Control,
  Address,
};
use crate::error::Result;
use super::BusComponent;


#[derive(Debug)]
pub struct ProgramCounter {
  control: Control,
  value: u16,
}

impl ProgramCounter {
  pub fn new() -> ProgramCounter {
    ProgramCounter {
      control: Control::new(),
      value: 0x0000,
    }
  }

  pub fn link(&self) -> u16 {
    self.value
  }
}

impl fmt::Display for ProgramCounter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.control.address {
      Address::ProgramCounter => write!(f, "[PC]")?,
      _ => write!(f, " PC ")?,
    }
    if self.control.pc.load && self.control.pc.out {
      write!(f, " <>")?;
    } else if self.control.pc.load {
      write!(f, " <=")?;
    } else if self.control.pc.out {
      write!(f, " =>")?;
    } else {
      write!(f, " ==")?;
    }
    write!(f, " 0x{:04X}", self.value)?;
    if self.control.pc.increment {
      write!(f, " ++")?;
    }
    Ok(())
  }
}

impl BusComponent for ProgramCounter {
  fn name(&self) -> &'static str {
    "ProgramCounter"
  }

  fn set_control(&mut self, control: Control) {
    self.control = control;

    if self.control.pc.increment {
      self.value = self.value.wrapping_add(1);
    }
  }

  fn load(&mut self, value: u16) -> Result<()> {
    if self.control.pc.load {
      self.value = value;
    }
    Ok(())
  }

  fn data(&self) -> Result<Option<u16>> {
    if self.control.pc.out {
      Ok(Some(self.value))
    } else {
      Ok(None)
    }
  }

  fn address(&self) -> Result<Option<u16>> {
    if let Address::ProgramCounter = self.control.address {
      Ok(Some(self.value))
    } else {
      Ok(None)
    }
  }
}
