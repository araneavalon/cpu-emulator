
use crate::control::Control;
use crate::components::BusComponent;


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
}

impl BusComponent for RegisterFile {
  fn set_control(&mut self, control: Control) {
    self.control = control;
  }

  fn load(&mut self, value: u16) {
    let register = self.control.register.load as usize;
    if register < self.values.len() {
      self.values[register] = value;
    }
  }

  fn data(&self) -> Option<u16> {
    let register = self.control.register.out as usize;
    if register < self.values.len() {
      Some(self.values[register])
    } else {
      None
    }
  }
}
