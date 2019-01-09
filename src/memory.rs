
use std::fmt;

use crate::bus;
use crate::control;
use crate::error::Error;


const MEM_SIZE: usize = 0x1FFFF;


pub struct Memory {
  control: control::Memory,
  ram: [u8; MEM_SIZE], // TODO BANK SWITCHING IN HERE YO
  addr: u16,
}

impl Memory {
  pub fn new() -> Memory {
    Memory {
      control: control::Memory::new(),
      ram: [0x00; MEM_SIZE], // TODO BANK SWITCHING
      addr: 0x0000,
    }
  }

  pub fn set_addr(&mut self, state: &bus::State) -> Result<(), Error> {
    match state.read_addr() {
      Err(error) => {
        if self.control.Data != control::ReadWrite::None {
          return Err(error)
        }
      },
      Ok(addr) => self.addr = addr,
    }
    Ok(())
  }
}

impl bus::Device<control::Memory> for Memory {
  fn update(&mut self, control: control::Memory) -> Result<(), Error> {
    self.control = control;
    Ok(())
  }

  fn read(&self) -> bus::State {
    bus::State {
      data: if let control::ReadWrite::Write = self.control.Data {
        Some(self.ram[self.addr as usize])
      } else {
        None
      },
      addr: None,
    }
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    if let control::ReadWrite::Read = self.control.Data {
      self.ram[self.addr as usize] = state.read_data()?;
    }
    Ok(())
  }
}

impl fmt::Display for Memory {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Memory({:#X} => Ram[0x({:X}){:X}])",
      self.addr, self.ram[(self.addr + 1) as usize], self.ram[self.addr as usize])
  }
}

impl fmt::Debug for Memory {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Memory({:#X} => Ram[0x({:X}){:X}] D={:?})",
      self.addr, self.ram[(self.addr + 1) as usize], self.ram[self.addr as usize], self.control.Data)
  }
}

impl PartialEq for Memory {
  fn eq(&self, other: &Memory) -> bool {
    self.control == other.control &&
      self.ram[..] == other.ram[..] &&
      self.addr == other.addr
  }

  fn ne(&self, other: &Memory) -> bool {
    self.control != other.control ||
      self.ram[..] != other.ram[..] ||
      self.addr != other.addr
  }
}

impl Eq for Memory {}
