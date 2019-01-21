
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use crate::error::Error;


const RAM_SIZE: usize = 0x20000; // 128K
const ROM_SIZE: usize = 0x04000; // 16K


pub struct Memory {
  ram: [u8; RAM_SIZE],
  rom: [u8; ROM_SIZE],
}

impl Memory {
  pub fn new() -> Memory {
    Memory {
      ram: [0x00; RAM_SIZE],
      rom: {
        let mut rom = [0x00; ROM_SIZE];

        let mut file = String::new();
        File::open("./rom.asm").unwrap().read_to_string(&mut file).unwrap();
        let binary = crate::assembler::assemble(&file).unwrap();
        for (address, byte) in binary.iter().enumerate() {
          rom[address] = *byte;
        }

        rom
      },
    }
  }

  pub fn get_rom(&self, address: usize) -> Result<u8, Error> {
    Ok(self.rom[address])
  }

  pub fn get_ram(&self, address: usize) -> Result<u8, Error> {
    Ok(self.ram[address])
  }

  pub fn set_ram(&self, address: usize, value: u8) -> Result<(), Error> {
    self.ram[address] = value;
    Ok(())
  }
}

impl PartialEq for Memory {
  fn eq(&self, other: &Memory) -> bool {
    self.ram[..] == other.ram[..] &&
      self.rom[..] == other.rom[..]
  }

  fn ne(&self, other: &Memory) -> bool {
    self.ram[..] != other.ram[..] ||
      self.rom[..] != other.rom[..]
  }
}

impl Eq for Memory {}

impl fmt::Display for Memory {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Memory {{No Display Implementation}}")
  }
}

impl fmt::Debug for Memory {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Memory {{No Debug Implementation}}")
  }
}
