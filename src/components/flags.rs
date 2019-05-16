
use std::fmt;
use crate::control::{
  Control,
  Condition,
};
use crate::error::Result;
use super::BusComponent;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Flag {
  Zero = 0,
  Sign,
  Carry,
  Overflow,

  InterruptZero = 8,
  InterruptOne,
  InterruptTwo,
  InterruptThree,
  InterruptFour,
  InterruptFive,
  InterruptSix,
  InterruptEnable,
}


#[derive(Debug)]
pub struct Flags {
  control: Control,
  flags: [bool; 16],
}

impl Flags {
  pub fn new() -> Flags {
    Flags {
      control: Control::new(),
      flags: [false; 16],
    }
  }

  pub fn test(&self, c: Control) -> bool {
    if let Some(interrupt) = c.branch.interrupt {
      return interrupt <= 7 && self.flags[Flag::InterruptEnable as usize] && !self.flags[(interrupt as usize) + 8]
    } else {
      c.branch.negate ^ match c.branch.condition {
        Condition::Always => true,
        Condition::Zero => self.flags[Flag::Zero as usize],
        Condition::Sign => self.flags[Flag::Sign as usize],
        Condition::Carry => self.flags[Flag::Carry as usize],
        Condition::CarryNotZero => self.flags[Flag::Carry as usize] && !self.flags[Flag::Zero as usize],
        Condition::Overflow => self.flags[Flag::Overflow as usize],
        Condition::OverflowNotZero => self.flags[Flag::Overflow as usize] && !self.flags[Flag::Zero as usize],
      }
    }
  }

  pub fn set(&mut self, flag: Flag, value: bool) {
    self.flags[flag as usize] = value;
  }

  pub fn set_alu(&mut self, flags: [bool; 8]) {
    if self.control.alu.set_flags {
      for i in 0..8 {
        self.flags[i] = flags[i];
      }
    }
  }

  fn value(&self) -> u16 {
    let mut value = 0;
    for i in 0..16 {
      value |= (self.flags[i] as u16) << i;
    }
    value
  }
}

impl fmt::Display for Flags {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.control.alu.set_flags {
      write!(f, " >F ")?;
    } else {
      write!(f, "  F ")?;
    }
    if self.control.flags.load && self.control.flags.out {
      write!(f, " <>")?;
    } else if self.control.flags.load {
      write!(f, " <=")?;
    } else if self.control.flags.out {
      write!(f, " =>")?;
    } else {
      write!(f, " ==")?;
    }
    write!(f, " 0b{:016b}", self.value())?;
    Ok(())
  }
}

impl BusComponent for Flags {
  fn name(&self) -> &'static str {
    "FlagsRegister"
  }

  fn set_control(&mut self, control: Control) {
    self.control = control;
  }

  fn load(&mut self, value: u16) -> Result<()> {
    if self.control.flags.load {
      for i in 0..16 {
        self.flags[i] = ((value >> i) & 1) != 0;
      }
    }
    Ok(())
  }

  fn data(&self) -> Result<Option<u16>> {
    if self.control.flags.out {
      Ok(Some(self.value()))
    } else {
      Ok(None)
    }
  }
}
