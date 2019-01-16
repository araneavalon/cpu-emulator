
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use crate::bus;
use crate::control;
use crate::error::Error;


pub const INTERRUPT_HANDLER: [u16; 2] = [0x0000, 0x0001];
pub const BREAK_HANDLER: [u16; 2] = [0x0002, 0x0003];
pub const BANK_ADDRESS: u16  = 0x0004;
pub const START_ADDRESS: u16 = 0x2000;


const RAM_SIZE: usize = 0x20000; // 128K
const ROM_SIZE: usize = 0x08000; // 32K

#[derive(Debug, PartialEq, Eq)]
enum Bank {
  Register,
  Zero(u16),
  One(u8, u16),
  Two(u8, u16),
  Three(u8, u16),
  Four(u16),
  Five(u8, u16),
  Six(u8, u16),
}

impl Bank {
  fn get(bank: u8, address: u16) -> Bank {
    if address == BANK_ADDRESS {
      Bank::Register
    } else if address <= 0x0FFF {
      Bank::Zero(address)
    } else if address <= 0x1FFF {
      Bank::One((bank >> 0) & 0b11, address)
    } else if address <= 0x3FFF {
      Bank::Two((bank >> 2) & 0b01, address)
    } else if address <= 0x5FFF {
      Bank::Three((bank >> 3) & 0b01, address)
    } else if address <= 0x7FFF {
      Bank::Four(address)
    } else if address <= 0xBFFF {
      Bank::Five((bank >> 4) & 0b11, address)
    } else {
      Bank::Six((bank >> 6) & 0b11, address)
    }
  }

  fn ram(&self) -> Option<usize> {
    match *self {
      Bank::Register             => None,
      Bank::Zero(address)        => Some(address as usize),
      Bank::One(0b11, address)   => Some(address as usize),
      Bank::Two(0b01, address)   => Some(address as usize),
      Bank::Three(0b00, address) => Some(address as usize),
      Bank::Four(address)        => Some(address as usize),
      Bank::Five(0b00, address)  => Some(address as usize),
      Bank::Five(0b01, address)  => Some(0x08000 + (address as usize)),
      Bank::Five(0b10, address)  => Some(0x10000 + (address as usize)),
      Bank::Six(0b00, address)   => Some(address as usize),
      Bank::Six(0b01, address)   => Some(0x0C000 + (address as usize)),
      Bank::Six(0b10, address)   => Some(0x14000 + (address as usize)),
      _                          => None,
    }
  }

  fn rom(&self) -> Option<usize> {
    match *self {
      Bank::Register             => None,
      Bank::One(0b01, address)   => Some(0x4000 + ((address & 0x0FFF) as usize)),
      Bank::One(0b10, address)   => Some(0x5000 + ((address & 0x0FFF) as usize)),
      Bank::Two(0b00, address)   => Some(0x0000 + ((address & 0x1FFF) as usize)),
      Bank::Three(0b01, address) => Some(0x2000 + ((address & 0x1FFF) as usize)),
      Bank::Five(0b11, address)  => Some(0x0000 + ((address & 0x1FFF) as usize)), // EXROM
      Bank::Six(0b11, address)   => Some(0x2000 + ((address & 0x1FFF) as usize)), // EXROM
      _                          => None,
    }
  }
}

// BANK SWITCHING
// 0   0000-0BFF  (3k) RAM
// 0   0C00-0FFF  (1k) Stack
// 1   1000-1FFF  (4k)    IO/Hardware | ROM | RAM
// 2   2000-3FFF  (8k)           Kernel ROM | RAM
// 3   4000-5FFF  (8k)                  ROM | RAM
// 4   6000-7FFF  (8k)    RAM
// 5   8000-BFFF (16k)    RAM | RAM | RAM
// 6   C000-FFFF (16k)    RAM | RAM | RAM

// 0x00-0xFF ; SYSTEM REGISTERS
//  0x0000  -> Bank Register
//  0x0001  -> 
// (0x0002) -> Interrupt Handler
// (0x0004) -> Break Handler

// Bank Register Bits (Somewhere in the IO/Hardware region)
// 7......0
// 66553211

// 1
// 00 IO/Hardware
// 01 ROM
// 10 ROM? (IDK WHAT TO PUT HERE YO)
// 11 RAM

// 23
// 0 ROM
// 1 RAM

// 5566
// 00 RAM 0   32- 48k /  48- 64k
// 01 RAM 1   64- 80k /  80- 96k
// 10 RAM 2   96-112k / 112-128k
// 11 EXROM    0- 16k /  16- 32k


pub struct Memory {
  control: control::Memory,
  address: u16,
  bank: u8,
  ram: [u8; RAM_SIZE],
  rom: [u8; ROM_SIZE],
}

impl Memory {
  pub fn new() -> Memory {
    let mut rom = [0x00; ROM_SIZE];

    // let mut file = String::new();
    // File::open(option_env!("ROM_FILE")).unwrap().read_to_string(&mut file).unwrap();
    // for (address, byte) in file.split_whitespace().enumerate() {
    //   rom[address] = u8::from_str_radix(byte, 16).unwrap();
    // }

    Memory {
      control: control::Memory::new(),
      address: 0x0000,
      bank: 0x00,
      ram: [0x00; RAM_SIZE],
      rom: rom,
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
    let bank = Bank::get(self.bank, self.address);
    if let Bank::Register = bank {
      Ok(self.bank)
    } else if let Some(address) = bank.rom() {
      Ok(self.rom[address])
    } else if let Some(address) = bank.ram() {
      Ok(self.ram[address])
    } else {
      Err(Error::Error(format!("Memory read failed, invalid location at address {:#X} (bank={:b}", self.address, self.bank)))
    }
  }

  fn set_value(&mut self, value: u8) -> Result<(), Error> {
    let bank = Bank::get(self.bank, self.address);
    if let Bank::Register = bank {
      self.bank = value;
    } else if let Some(_) = bank.rom() {
      return Err(Error::InvalidWrite(format!("Can not write to Read Only Memory at address {:#X} (bank={:b})", self.address, self.bank)));
    } else if let Some(address) = bank.ram() {
      self.ram[address] = value;
    }
    Ok(())
  }
}

impl bus::Device<control::Memory> for Memory {
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

impl PartialEq for Memory {
  fn eq(&self, other: &Memory) -> bool {
    self.control == other.control &&
      self.ram[..] == other.ram[..] &&
      self.rom[..] == other.rom[..] &&
      self.bank == other.bank &&
      self.address == other.address
  }

  fn ne(&self, other: &Memory) -> bool {
    self.control != other.control ||
      self.ram[..] != other.ram[..] ||
      self.rom[..] != other.rom[..] ||
      self.bank != other.bank ||
      self.address != other.address
  }
}

impl Eq for Memory {}

impl fmt::Display for Memory {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let bank = Bank::get(self.bank, self.address);
    let (value, source) = if let Bank::Register = bank {
      (self.bank, String::from("BANK[     ]"))
    } else if let Some(address) = bank.ram() {
      (self.ram[address], format!("RAM[0x{:04X}]", address))
    } else if let Some(address) = bank.rom() {
      (self.rom[address], format!("ROM[0x{:04X}]", address))
    } else {
      (0, String::from("UNKNOWN[  ]"))
    };

    write!(f, "0x  {:02X} <= 0x{:04X} {} (Data={}, Bank=0b{:08b}) [Memory]", value, self.address, source, self.control.Data, self.bank)
  }
}

impl fmt::Debug for Memory {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Memory {{No Debug Implementation}}")
  }
}
