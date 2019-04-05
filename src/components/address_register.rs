
use crate::control::{
  Control,
  Address,
};
use crate::components::BusComponent;


#[derive(Debug)]
pub struct AddressRegister {
  control: Control,
  value: u16,
}

impl AddressRegister {
  pub fn new() -> AddressRegister {
    AddressRegister {
      control: Control::new(),
      value: 0x0000,
    }
  }
}

impl BusComponent for AddressRegister {
  fn set_control(&mut self, control: Control) {
    self.control = control;
  }

  fn load(&mut self, value: u16) {
    if self.control.a.load {
      self.value = value;
    }
  }

  fn address(&self) -> Option<u16> {
    if let Address::A = self.control.address {
      Some(self.value)
    } else {
      None
    }
  }
}
