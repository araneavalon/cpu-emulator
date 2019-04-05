
use crate::control::{
  Control,
  IMode,
};
use crate::components::BusComponent;


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
}

impl BusComponent for InstructionRegister {
  fn set_control(&mut self, control: Control) {
    self.control = control;
  }

  fn load(&mut self, value: u16) {
    if self.control.pc.load {
      self.value = value;
    }
  }

  fn data(&self) -> Option<u16> {
    match self.control.i.mode {
      IMode::SignedByte   => Some((((self.value >> 3) & 0x00FF) as i8) as u16),
      IMode::UnsignedByte => Some((self.value >> 3) & 0x00FF),
      IMode::WordOffset   => Some((self.value >> 2) & 0x000E),
      IMode::Bitmask      => Some(1 << ((self.value >> 3) & 0x001F)),
      IMode::Interrupt    => Some(0xFFF8 | ((self.value >> 3) & 0x0007)),
      IMode::None         => None,
    }
  }
}
