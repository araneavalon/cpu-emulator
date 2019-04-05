
use crate::control::{
  Control,
  Address,
};
use crate::components::BusComponent;


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

impl BusComponent for ProgramCounter {
  fn set_control(&mut self, control: Control) {
    self.control = control;

    if self.control.pc.increment {
      if self.value != 0xFFFF {
        self.value += 1;
      } else {
        self.value = 0x0000;
      }
    }
  }

  fn load(&mut self, value: u16) {
    if self.control.pc.load {
      self.value = value;
    }
  }

  fn data(&self) -> Option<u16> {
    if self.control.pc.out {
      Some(self.value)
    } else {
      None
    }
  }

  fn address(&self) -> Option<u16> {
    if let Address::ProgramCounter = self.control.address {
      Some(self.value)
    } else {
      None
    }
  }
}
