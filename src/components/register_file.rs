
use std::fmt;
use crate::control::{
  Control,
  Register,
};
use crate::error::Result;
use super::BusComponent;


#[derive(Debug)]
pub struct RegisterFile {
  control: Control,
  values: [u16; 8],
}

impl RegisterFile {
  pub fn new() -> RegisterFile {
    RegisterFile {
      control: Control::new(),
      values: [0x0000; 8],
    }
  }

  fn fmt_r(&self, f: &mut fmt::Formatter, r: Register, name: &'static str) -> fmt::Result {
    let load = self.control.register.load;
    let out = self.control.register.out;
    if (r == load) && (r == out) {
      write!(f, "  {}  <> 0x{:04X}", name, self.values[r as usize])
    } else if r == load {
      write!(f, "  {}  <= 0x{:04X}", name, self.values[r as usize])
    } else if r == out {
      write!(f, "  {}  => 0x{:04X}", name, self.values[r as usize])
    } else {
      write!(f, "  {}  == 0x{:04X}", name, self.values[r as usize])
    }
  }
}

impl fmt::Display for RegisterFile {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.fmt_r(f, Register::Zero, "A")?;
    write!(f, "\n")?;
    self.fmt_r(f, Register::One, "B")?;
    write!(f, "\n")?;
    self.fmt_r(f, Register::Two, "C")?;
    write!(f, "\n")?;
    self.fmt_r(f, Register::Three, "D")?;
    write!(f, "\n")?;
    self.fmt_r(f, Register::Four, "E")?;
    write!(f, "\n")?;
    self.fmt_r(f, Register::Five, "X")?;
    write!(f, "\n")?;
    self.fmt_r(f, Register::Six, "Y")?;
    write!(f, "\n")?;
    self.fmt_r(f, Register::Seven, "Z")?;
    Ok(())
  }
}

impl BusComponent for RegisterFile {
  fn name(&self) -> &'static str {
    "RegisterFile"
  }

  fn set_control(&mut self, control: Control) {
    self.control = control;
  }

  fn load(&mut self, value: u16) -> Result<()> {
    let register = self.control.register.load as usize;
    if register < self.values.len() {
      self.values[register] = value;
    }
    Ok(())
  }

  fn data(&self) -> Result<Option<u16>> {
    let register = self.control.register.out as usize;
    if register < self.values.len() {
      Ok(Some(self.values[register]))
    } else {
      Ok(None)
    }
  }
}
