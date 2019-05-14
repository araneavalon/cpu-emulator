
use std::fmt;
use crate::control::{
  Control,
  Address,
};
use crate::error::Result;
use super::BusComponent;


#[derive(Debug)]
pub struct StackPointers {
  control: Control,
  values: [u16; 2],
}

impl StackPointers {
  pub fn new() -> StackPointers {
    StackPointers {
      control: Control::new(),
      values: [0x0000; 2],
    }
  }

  fn fmt_s(&self, f: &mut fmt::Formatter, s: usize) -> fmt::Result {
    match self.control.address {
      Address::StackZero if s == 0 => write!(f, "[S0]")?,
      Address::StackOne if s == 1 => write!(f, "[S1]")?,
      _ => write!(f, " S{} ", s)?,
    }
    if self.control.s[s].load && self.control.s[s].out {
      write!(f, " <>")?;
    } else if self.control.s[s].load {
      write!(f, " <=")?;
    } else if self.control.s[s].out {
      write!(f, " =>")?;
    } else {
      write!(f, " ==")?;
    }
    write!(f, " 0x{:04X}", self.values[s])?;
    if self.control.s[s].count {
      if self.control.s[s].direction {
        write!(f, " ++")?;
      } else {
        write!(f, " --")?;
      }
    }
    Ok(())
  }
}

impl fmt::Display for StackPointers {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.fmt_s(f, 0)?;
    write!(f, "\n")?;
    self.fmt_s(f, 1)?;
    Ok(())
  }
}

impl BusComponent for StackPointers {
  fn name(&self) -> &'static str {
    "StackPointers"
  }

  fn set_control(&mut self, control: Control) {
    self.control = control;

    for i in 0..2 {
      if self.control.s[i].count {
        if self.control.s[i].direction {
          self.values[i] = self.values[i].wrapping_add(1);
        } else {
          self.values[i] = self.values[i].wrapping_sub(1);
        }
      }
    }
  }

  fn load(&mut self, value: u16) -> Result<()> {
    for i in 0..2 {
      if self.control.s[i].load {
        self.values[i] = value;
      }
    }
    Ok(())
  }

  fn data(&self) -> Result<Option<u16>> {
    for i in 0..2 {
      if self.control.s[i].out {
        return Ok(Some(self.values[i]))
      }
    }
    Ok(None)
  }

  fn address(&self) -> Result<Option<u16>> {
    match self.control.address {
      Address::StackZero => Ok(Some(self.values[0])),
      Address::StackOne => Ok(Some(self.values[1])),
      _ => Ok(None),
    }
  }
}
