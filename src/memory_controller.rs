
use std::fmt;

use crate::bus;
use crate::control;
use crate::error::Error;

use crate::components::memory::Memory;
use crate::io::Io;


// BANK SWITCHING
// 0   0000-03FF  (1k)               RAM
// 0   0400-07FF  (1k)               Stack
// 0   0800-1FFF  (6k)               RAM
// 1   2000-3FFF  (8k)         Video RAM / IO Ports
// 2   4000-5FFF  (8k)  Kernel ROM | RAM
// 3   6000-7FFF  (8k)   Forth ROM | RAM
// 4   8000-BFFF (16k)   RAM | RAM | RAM | EXROM
// 5   C000-FFFF (16k)   RAM | RAM | RAM | EXROM

// Bank Register Bits (Somewhere in the IO/Hardware region)
// 7......0
// 55544421

// 12
// 0 ROM
// 1 RAM

// 444555
// 000 RAM 0   32- 48k /  48- 64k
// 001 RAM 1   64- 80k /  80- 96k
// 010 RAM 2   96-112k / 112-128k
// 011
// 1bx EXROM Bank bL / bH

// Video Ram
// 0x6000-0x67FF Character (256 8x8)
// 0x6800-0x6FFF Text (2 screens)
// 0x7000-0x7EFF Graphics (1 screen)

// IO Ports
// 0x1F00-0x1FFF


pub const INTERRUPT_HANDLER: [u16; 2] = [0x0000, 0x0001];
pub const BREAK_HANDLER: [u16; 2] = [0x0002, 0x0003];
pub const BANK_ADDRESS: u16  = 0x7F00;
pub const START_ADDRESS: u16 = 0x4000;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Bank {
  Register,
  Ram(usize),
  Rom(usize),
  Io(u16),
  Exrom(usize),
  Invalid,
}

impl Bank {
  fn get(bank: u8, address: u16) -> Bank {
    let address: usize = address as usize;

    if address == (BANK_ADDRESS as usize) {
      Bank::Register
    } else if address <= 0x1FFF {
      Bank::Ram(address)
    } else if address <= 0x3FFF {
      Bank::Io((address as u16) & 0x1FFF)
    } else if address <= 0x5FFF {
      match (bank >> 0) & 0b1 {
        0b0 => Bank::Rom(0x0000 + (address & 0x1FFF)),
        0b1 => Bank::Ram(address),
        _ => panic!("This is literally impossible."),
      }
    } else if address <= 0x7FFF {
      match (bank >> 1) & 0b1 {
        0b0 => Bank::Rom(0x2000 + (address & 0x1FFF)),
        0b1 => Bank::Ram(address),
        _ => panic!("This is literally impossible."),
      }
    } else if address <= 0xBFFF {
      match (bank >> 2) & 0b111 {
        0b000 => Bank::Ram(0x08000 + (address & 0x3FFF)),
        0b010 => Bank::Ram(0x10000 + (address & 0x3FFF)),
        0b001 => Bank::Ram(0x18000 + (address & 0x3FFF)),
        0b011 => Bank::Invalid,
        0b100 => Bank::Exrom(0x0000 + (address & 0x3FFF)),
        0b101 => Bank::Exrom(0x4000 + (address & 0x3FFF)),
        0b110 => Bank::Exrom(0x8000 + (address & 0x3FFF)),
        0b111 => Bank::Exrom(0xC000 + (address & 0x3FFF)),
        _ => panic!("This is literally impossible."),
      }
    } else {
      match (bank >> 5) & 0b111 {
        0b000 => Bank::Ram(0x0C000 + (address & 0x3FFF)),
        0b010 => Bank::Ram(0x14000 + (address & 0x3FFF)),
        0b001 => Bank::Ram(0x1C000 + (address & 0x3FFF)),
        0b011 => Bank::Invalid,
        0b100 => Bank::Exrom(0x4000 + (address & 0x3FFF)),
        0b101 => Bank::Exrom(0x0000 + (address & 0x3FFF)),
        0b110 => Bank::Exrom(0xC000 + (address & 0x3FFF)),
        0b111 => Bank::Exrom(0x8000 + (address & 0x3FFF)),
        _ => panic!("This is literally impossible."),
      }
    }
  }
}


#[derive(PartialEq, Eq)]
pub struct MemoryController {
  control: control::Memory,
  address: u16,
  bank: u8,
  memory: Memory,
  pub io: Io,
}

impl MemoryController {
  pub fn new() -> MemoryController {
    MemoryController {
      control: control::Memory::new(),
      address: 0x0000,
      bank: 0x00,
      memory: Memory::new(),
      io: Io::new(),
    }
  }

  pub fn set_addr(&mut self, state: &bus::State) -> Result<(), Error> {
    match state.read_addr() {
      Err(error) => {
        if self.control.Data != control::ReadWrite::None {
          return Err(error)
        }
      },
      Ok(address) => self.address = address,
    }
    Ok(())
  }

  fn get_value(&self) -> Result<u8, Error> {
    match Bank::get(self.bank, self.address) {
      Bank::Register => Ok(self.bank),
      Bank::Io(address) => Ok(self.io.get_value(address)?),
      Bank::Ram(address) => Ok(self.memory.get_ram(address)?),
      Bank::Rom(address) => Ok(self.memory.get_rom(address)?),
      Bank::Exrom(_address) => Err(Error::InvalidRead(String::from("EXROM not implemented yet."))),
      Bank::Invalid => Err(Error::InvalidRead(format!("Can not read from address 0x{:04X} (bank={:08b})", self.address, self.bank))),
    }
  }

  fn set_value(&mut self, value: u8) -> Result<(), Error> {
    match Bank::get(self.bank, self.address) {
      Bank::Register => self.bank = value,
      Bank::Io(address) => self.io.set_value(address, value)?,
      Bank::Ram(address) => self.memory.set_ram(address, value)?,
      Bank::Rom(address) => return Err(
        Error::InvalidWrite(format!(
          "Can not write to Read Only Memory at address 0x{:04X} (0x{:04X}) (bank={:08b})",
          self.address, address, self.bank
        ))
      ),
      Bank::Exrom(_address) => return Err(Error::InvalidWrite(String::from("EXROM not implemented yet."))),
      Bank::Invalid => return Err(Error::InvalidWrite(format!("Can not write to address 0x{:04X} (bank={:08b})", self.address, self.bank))),
    }
    Ok(())
  }
}

impl bus::Device<control::Memory> for MemoryController {
  fn update(&mut self, control: control::Memory) -> Result<(), Error> {
    self.control = control;
    Ok(())
  }

  fn read(&self) -> Result<bus::State, Error> {
    Ok(bus::State {
      data: if let control::ReadWrite::Write = self.control.Data {
        Some(self.get_value()?)
      } else {
        None
      },
      addr: None,
    })
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    if let control::ReadWrite::Read = self.control.Data {
      self.set_value(state.read_data()?)?;
    }
    Ok(())
  }
}

impl fmt::Display for MemoryController {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let (value, source) = match Bank::get(self.bank, self.address) {
      Bank::Register => (self.bank, String::from("BANK[     ]")),
      Bank::Io(address) => (self.io.get_value(address).unwrap(), format!(" IO[0x{:04X}]", address)),
      Bank::Ram(address) => (self.memory.get_ram(address).unwrap(), format!("RAM[0x{:04X}]", address)),
      Bank::Rom(address) => (self.memory.get_rom(address).unwrap(), format!("ROM[0x{:04X}]", address)),
      Bank::Exrom(_address) => (0x00, String::from("EXROM[ BAD]")),
      Bank::Invalid => (0x00, String::from("INVALID[--]")),
    };
    write!(f, "0x  {:02X} <= 0x{:04X} {} Bank=0b{:08b}", value, self.address, source, self.bank)
  }
}

impl fmt::Debug for MemoryController {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let (value, source) = match Bank::get(self.bank, self.address) {
      Bank::Register => (self.bank, String::from("BANK[     ]")),
      Bank::Io(address) => (self.io.get_value(address).unwrap(), format!(" IO[0x{:04X}]", address)),
      Bank::Ram(address) => (self.memory.get_ram(address).unwrap(), format!("RAM[0x{:04X}]", address)),
      Bank::Rom(address) => (self.memory.get_rom(address).unwrap(), format!("ROM[0x{:04X}]", address)),
      Bank::Exrom(_address) => (0x00, String::from("EXROM[ BAD]")),
      Bank::Invalid => (0x00, String::from("INVALID[--]")),
    };
    write!(f, "0x  {:02X} <= 0x{:04X} {} (Data={:?}, Bank=0b{:08b}) [Memory]", value, self.address, source, self.control.Data, self.bank)
  }
}
