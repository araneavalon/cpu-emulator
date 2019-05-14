
use std::ops::{Index, IndexMut};
use std::fmt;

use crate::control::Control;
use crate::components::BusComponent;
use crate::io::{
  Io,
  Screen,
};
use crate::error::{
  Result,
  Error,
};

mod ram;
mod rom;

use ram::Ram;
use rom::Rom;


// 8k IO/VRAM
// 8k KERNEL/FORTH?

// 0x0000 0x00FF   0.25k  Zero Page
// 0x0100 0x03FF   0.75k
// 0x0400 0x05FF   0.5k   S0 Default
// 0x0600 0x07FF   0.5k   S1 Default
// 0x0800 0x0FFF   2k
// 0x1000 0x1FFF   4k

// 0xC000 0xCBFF   Text (3 screens) (only uses low byte)
// 0xCC00 0xCFFF   Character (256 8x8)
// 0xD000 0xDDFF   Graphics (2 screens)
// 0xDE00 0xDFFF   IO Ports (512)

// 0x0000 0x1FFF   SYSTEM/ZEROPAGE/ETC
// 0x2000 0x3FFF
// 0x4000 0x5FFF
// 0x6000 0x7FFF
// 0x8000 0x9FFF
// 0xA000 0xBFFF
// 0xC000 0xDFFF   IO/VRAM
// 0xE000 0xFFFF   KERNEL/FORTH

// SCREEN:
// https://www.mouser.com/datasheet/2/291/NHD-240128WG-ATFH-VZ-27453.pdf
// http://www.newhavendisplay.com/app_notes/RA6963.pdf
// LEVEL SHIFTER:
// http://www.ti.com/lit/ds/symlink/txb0108.pdf
// IO CONTROLLER:
// https://www.st.com/resource/en/datasheet/stm32f072c8.pdf



#[derive(Debug)]
pub struct Memory {
  control: Control,
  address: u16,
  ram: Ram,
  rom: Rom,
  io: Io,
}

impl Memory {
  pub fn new(rom: Vec<u16>) -> Memory {
    Memory {
      control: Control::new(),
      address: 0x0000,
      ram: Ram::new(),
      rom: Rom::new(rom),
      io: Io::new(),
    }
  }

  pub fn set_address(&mut self, address: u16) {
    self.address = address;
  }

  pub fn screen(&self) -> Result<&Screen> {
    self.io.screen()
  }

  fn component(&self, address: u16) -> Result<&dyn Index<u16, Output = u16>> {
    if self.ram.valid(address) {
      Ok(&self.ram)
    } else if self.rom.valid(address) {
      Ok(&self.rom)
    } else if self.io.valid(address) {
      Ok(&self.io)
    } else {
      Err(Error::InvalidRead(address, "No component available at address. This should be Impossible."))
    }
  }

  fn component_mut(&mut self, address: u16) -> Result<&mut dyn IndexMut<u16, Output = u16>> {
    if self.ram.valid(address) {
      Ok(&mut self.ram)
    } else if self.rom.valid(address) {
      Err(Error::InvalidWrite(address, "Can not write to ROM."))
    } else if self.io.valid(address) {
      Ok(&mut self.io)
    } else {
      Err(Error::InvalidWrite(address, "No component available at address. This should be Impossible."))
    }
  }

  fn name(&self, address: u16) -> Result<&'static str> {
    if self.ram.valid(address) {
      Ok(self.ram.name())
    } else if self.rom.valid(address) {
      Ok(self.rom.name())
    } else if self.io.valid(address) {
      Ok(self.io.name())
    } else {
      Err(Error::Impossible(address, "No component available at address. This should be Impossible."))
    }
  }
}

impl BusComponent for Memory {
  fn name(&self) -> &'static str {
    "Memory"
  }

  fn set_control(&mut self, control: Control) {
    self.control = control;
  }

  fn load(&mut self, value: u16) -> Result<()> {
    if self.control.memory.load {
      let address = self.address;
      print!("Memory Write: {}:={}", self.name(address)?, value);
      self.component_mut(address)?[address] = value;
      println!(" => {}", self.component(address)?[address]);
    }
    Ok(())
  }

  fn data(&self) -> Result<Option<u16>> {
    if self.control.memory.out {
      let address = self.address;
      Ok(Some(self.component(address)?[address]))
    } else {
      Ok(None)
    }
  }
}

impl fmt::Display for Memory {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.control.memory.load && self.control.memory.out {
      write!(f, " MEM <>")?;
    } else if self.control.memory.load {
      write!(f, " MEM <=")?;
    } else if self.control.memory.out {
      write!(f, " MEM =>")?;
    } else {
      write!(f, " MEM ==")?;
    }
    let address = self.address;
    let value = self.component(address)?[address];
    write!(f, " 0x{:04X} <- {}[0x{:04X}]", value, self.name(address)?, address)?;
    Ok(())
  }
}
