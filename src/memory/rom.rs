
use std::ops::Index;
use std::fmt;


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

  pub fn name(&self) -> &'static str {
    "ROM"
  }

  pub fn valid(&self, address: u16) -> bool {
    (ROM_OFFSET <= (address as usize)) && ((address as usize) < (ROM_SIZE + ROM_OFFSET))
  }
}

impl Index<u16> for Rom {
  type Output = u16;

  fn index(&self, address: u16) -> &u16 {
    &self.data[(address as usize) - ROM_OFFSET]
  }
}

impl fmt::Debug for Rom {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(&self.data[..], f)
  }
}
