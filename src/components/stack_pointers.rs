
use crate::control::{
  Control,
  Address,
};
use crate::components::BusComponent;


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
}

impl BusComponent for StackPointers {
  fn set_control(&mut self, control: Control) {
    self.control = control;

    for i in 0..2 {
      if self.control.s[i].count {
        if self.control.s[i].direction {
          if self.values[i] != 0xFFFF {
            self.values[i] += 1;
          } else {
            self.values[i] = 0x0000;
          }
        } else {
          if self.values[i] != 0x0000 {
            self.values[i] -= 1;
          } else {
            self.values[i] = 0xFFFF;
          }
        }
      }
    }
  }

  fn load(&mut self, value: u16) {
    for i in 0..2 {
      if self.control.s[i].load {
        self.values[i] = value;
      }
    }
  }

  fn data(&self) -> Option<u16> {
    for i in 0..2 {
      if self.control.s[i].out {
        return Some(self.values[i])
      }
    }
    None
  }

  fn address(&self) -> Option<u16> {
    match self.control.address {
      Address::StackZero => Some(self.values[0]),
      Address::StackOne => Some(self.values[1]),
      _ => None,
    }
  }
}
