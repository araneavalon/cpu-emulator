
use std::ops::{Index, IndexMut};
use std::fmt;


const RAM_SIZE: usize   = 0xC000;
const RAM_OFFSET: usize = 0x0000;

pub struct Ram {
  data: [u16; RAM_SIZE],
}

impl Ram {
  pub fn new() -> Ram {
    Ram { data: [0x0000; RAM_SIZE] }
  }

  pub fn name(&self) -> &'static str {
    "RAM"
  }

  pub fn valid(&self, address: u16) -> bool {
    (RAM_OFFSET <= (address as usize)) && ((address as usize) < (RAM_SIZE + RAM_OFFSET))
  }
}

impl Index<u16> for Ram {
  type Output = u16;

  fn index(&self, address: u16) -> &u16 {
    &self.data[(address as usize) - RAM_OFFSET]
  }
}

impl IndexMut<u16> for Ram {
  fn index_mut(&mut self, address: u16) -> &mut u16 {
    &mut self.data[(address as usize) - RAM_OFFSET]
  }
}

impl fmt::Debug for Ram {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(&self.data[..], f)
  }
}
