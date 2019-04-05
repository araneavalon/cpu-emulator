
use std::fmt;
use std::ops::{Index, IndexMut};

use crate::control::Control;
use crate::components::BusComponent;


const RAM_WORDS: usize = 2048;

struct Ram {
  data: [u16; RAM_WORDS],
}

impl Ram {
  pub fn new() -> Ram {
    Ram { data: [0x0000; RAM_WORDS] }
  }
}

impl fmt::Debug for Ram {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(&self.data[..], f)
  }
}

impl Index<u16> for Ram {
  type Output = u16;

  fn index(&self, address: u16) -> &u16 {
    &self.data[address as usize]
  }
}

impl IndexMut<u16> for Ram {
  fn index_mut(&mut self, address: u16) -> &mut u16 {
    &mut self.data[address as usize]
  }
}


#[derive(Debug)]
pub struct Memory {
  control: Control,
  address: u16,
  ram: Ram,
}

impl Memory {
  pub fn new() -> Memory {
    Memory {
      control: Control::new(),
      address: 0x0000,
      ram: Ram::new(),
    }
  }

  pub fn set_address(&mut self, address: u16) {
    self.address = address;
  }
}

impl BusComponent for Memory {
  fn set_control(&mut self, control: Control) {
    self.control = control;
  }

  fn load(&mut self, value: u16) {
    if self.control.memory.load {
      let index = self.address >> 1;
      if self.control.memory.word {
        self.ram[index] = value;
      } else if (self.address & 1) != 0 {
        self.ram[index] = (self.ram[index] & 0x00FF) | (value << 8);
      } else {
        self.ram[index] = (self.ram[index] & 0xFF00) | (value & 0x00FF);
      }
    }
  }

  fn data(&self) -> Option<u16> {
    if !self.control.memory.out {
      let index = self.address >> 1;
      if self.control.memory.word {
        Some(self.ram[index])
      } else if (self.address & 1) != 0 {
        Some(self.ram[index] >> 8)
      } else {
        Some(self.ram[index] & 0x00FF)
      }
    } else {
      None
    }
  }
}
