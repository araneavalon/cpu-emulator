
use crate::control::{
  Control,
  Condition,
};
use crate::components::BusComponent;


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
  InterruptMask,
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

  pub fn test(&self, negate: bool, condition: Condition) -> bool {
    negate ^ match condition {
      Condition::Always => true,
      Condition::Zero => self.flags[Flag::Zero as usize],
      Condition::Sign => self.flags[Flag::Sign as usize],
      Condition::Carry => self.flags[Flag::Carry as usize],
      Condition::CarryNotZero => self.flags[Flag::Carry as usize] & !self.flags[Flag::Zero as usize],
      Condition::Overflow => self.flags[Flag::Overflow as usize],
      Condition::OverflowNotZero => self.flags[Flag::Overflow as usize] & !self.flags[Flag::Zero as usize],
    }
  }

  pub fn set(&mut self, flags: [bool; 8]) {
    if self.control.alu.set_flags {
      for i in 0..8 {
        self.flags[i] = flags[i];
      }
    }
  }
}

impl BusComponent for Flags {
  fn set_control(&mut self, control: Control) {
    self.control = control;
  }

  fn load(&mut self, value: u16) {
    if self.control.flags.load {
      for i in 0..16 {
        self.flags[i] = ((value >> i) & 1) != 0;
      }
    }
  }

  fn data(&self) -> Option<u16> {
    if self.control.flags.out {
      let mut out = 0;
      for i in 0..16 {
        out = out | ((self.flags[i] as u16) << i);
      }
      Some(out)
    } else {
      None
    }
  }
}
