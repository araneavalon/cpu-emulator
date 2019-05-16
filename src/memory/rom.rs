
use std::fmt;
use super::Addressable;
use crate::error::{
  Result,
  Error,
};


const ROM_SIZE: usize   = 0x2000;
const ROM_OFFSET: usize = 0xE000;

pub struct Rom {
  data: [u16; ROM_SIZE],
}

impl Rom {
  pub fn new(rom: Vec<u16>) -> Rom {
    let mut data = [0x0000; ROM_SIZE];
    for (address, word) in rom.into_iter().enumerate() {
      data[address] = word;
    }
    Rom { data }
  }
}

impl Addressable for Rom {
  fn name(&self) -> &'static str {
    "ROM"
  }

  fn valid(&self, address: u16) -> bool {
    (ROM_OFFSET <= (address as usize)) && ((address as usize) < (ROM_SIZE + ROM_OFFSET))
  }

  fn read(&self, address: u16) -> Result<u16> {
    Ok(self.data[(address as usize) - ROM_OFFSET])
  }

  fn write(&mut self, address: u16, _: u16) -> Result<()> {
    Err(Error::InvalidWrite(address, "Unable to write ROM."))
  }
}

impl fmt::Debug for Rom {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(&self.data[..], f)
  }
}
