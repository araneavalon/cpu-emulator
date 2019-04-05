
use crate::control::{
  Control,
  Address,
};
use crate::components::BusComponent;


#[derive(Debug)]
pub struct LinkRegister {
  control: Control,
  value: u16,
}

impl LinkRegister {
  pub fn new() -> LinkRegister {
    LinkRegister {
      control: Control::new(),
      value: 0x0000,
    }
  }

  pub fn link(&mut self, value: u16) {
    if self.control.link {
      self.value = value;
    }
  }
}

impl BusComponent for LinkRegister {
  fn set_control(&mut self, control: Control) {
    self.control = control;

    if self.control.lr.increment {
      if self.value != 0xFFFF {
        self.value += 1;
      } else {
        self.value = 0x0000;
      }
    }
  }

  fn load(&mut self, value: u16) {
    if self.control.lr.load {
      self.value = value;
    }
  }

  fn data(&self) -> Option<u16> {
    if self.control.lr.out {
      Some(self.value)
    } else {
      None
    }
  }
}
