
use std::fmt;
use super::Addressable;
use crate::error::Result;


const RAM_SIZE: usize   = 0xC000;
const RAM_OFFSET: usize = 0x0000;

pub struct Ram {
  data: [u16; RAM_SIZE],
}

impl Ram {
  pub fn new() -> Ram {
    Ram { data: [0x0000; RAM_SIZE] }
  }
}

impl Addressable for Ram {
  fn name(&self) -> &'static str {
    "RAM"
  }

  fn valid(&self, address: u16) -> bool {
    (RAM_OFFSET <= (address as usize)) && ((address as usize) < (RAM_SIZE + RAM_OFFSET))
  }

  fn read(&self, address: u16) -> Result<u16> {
    Ok(self.data[(address as usize) - RAM_OFFSET])
  }

  fn write(&mut self, address: u16, value: u16) -> Result<()> {
    self.data[(address as usize) - RAM_OFFSET] = value;
    Ok(())
  }
}

impl fmt::Debug for Ram {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(&self.data[..], f)
  }
}
