
use std::fmt;
use crate::control::{
  Control,
  IMode,
};
use crate::error::Result;
use super::BusComponent;


#[derive(Debug)]
pub struct InstructionRegister {
  control: Control,
  value: u16,
}

impl InstructionRegister {
  pub fn new() -> InstructionRegister {
    InstructionRegister {
      control: Control::new(),
      value: 0x0000,
    }
  }

  pub fn get(&self) -> u16 {
    self.value
  }

  pub fn set(&mut self, value: u16) {
    self.value = value;
  }
}

impl fmt::Display for InstructionRegister {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.control.i.load {
      write!(f, "  I  <")?;
    } else {
      write!(f, "  I  =")?;
    }
    match (self.control.i.mode, self.data()?) {
      (IMode::SignedByte,   Some(b)) => write!(f, "> 0x{:04X} Signed({})", self.value, b)?,
      (IMode::UnsignedByte, Some(u)) => write!(f, "> 0x{:04X} Unsigned(0x{:04X})", self.value, u)?,
      (IMode::Bitmask,      Some(m)) => write!(f, "> 0x{:04X} Mask(0b{:016b})", self.value, m)?,
      (IMode::Interrupt,    Some(i)) => write!(f, "> 0x{:04X} Interrupt({})", self.value, i)?,
      (IMode::Startup,      Some(a)) => write!(f, "> 0x{:04X} Startup(0x{:04X})", self.value, a)?,
      _ => write!(f, "= 0x{:04X}", self.value)?,
    }
    Ok(())
  }
}

impl BusComponent for InstructionRegister {
  fn name(&self) -> &'static str {
    "InstructionRegister"
  }

  fn set_control(&mut self, control: Control) {
    self.control = control;
  }

  fn load(&mut self, value: u16) -> Result<()> {
    if self.control.i.load {
      self.value = value;
    }
    Ok(())
  }

  fn data(&self) -> Result<Option<u16>> {
    match self.control.i.mode {
      IMode::SignedByte   => Ok(Some((((self.value >> 3) & 0x00FF) as i8) as u16)),
      IMode::UnsignedByte => Ok(Some((self.value >> 3) & 0x00FF)),
      IMode::Bitmask      => Ok(Some(1 << ((self.value >> 3) & 0x000F))),
      IMode::Interrupt    => Ok(Some(0xFFF8 | ((self.value >> 3) & 0x0007))), // TODO put interrupts in the right place.
      IMode::Startup      => Ok(Some(0xFFFF)),
      IMode::None         => Ok(None),
    }
  }
}
