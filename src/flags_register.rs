
use std::fmt;

use crate::bus;
use crate::control;
use crate::error::Error;


const Z_BIT: u8 = 0;
const C_BIT: u8 = 1;
const V_BIT: u8 = 2;
const S_BIT: u8 = 3;
const I_BIT: u8 = 7;


#[derive(PartialEq, Eq)]
pub struct FlagsRegister {
  control: control::FlagsRegister,
  flags: control::Flags,
}

impl FlagsRegister {
  pub fn new() -> FlagsRegister {
    FlagsRegister {
      control: control::FlagsRegister::new(),
      flags: hash_map!{
        control::Flag::Z => false,
        control::Flag::C => false,
        control::Flag::V => false,
        control::Flag::S => false,
        control::Flag::I => false,
      },
    }
  }

  pub fn get_flags(&self) -> &control::Flags {
    &self.flags
  }

  pub fn set_flags(&mut self, flags: control::Flags) {
    if let control::Read::Read = self.control.Update {
      self.flags.extend(flags);
    }
  }

  fn to_value(&self) -> u8 {
    0x00 |
      (self.flags[&control::Flag::Z] as u8) << Z_BIT |
      (self.flags[&control::Flag::C] as u8) << C_BIT |
      (self.flags[&control::Flag::V] as u8) << V_BIT |
      (self.flags[&control::Flag::S] as u8) << S_BIT |
      (self.flags[&control::Flag::I] as u8) << I_BIT
  }

  fn from_value(&mut self, value: u8) {
    self.flags.insert(control::Flag::Z, (value & (0x01 << Z_BIT)) != 0);
    self.flags.insert(control::Flag::C, (value & (0x01 << C_BIT)) != 0);
    self.flags.insert(control::Flag::V, (value & (0x01 << V_BIT)) != 0);
    self.flags.insert(control::Flag::S, (value & (0x01 << S_BIT)) != 0);
    self.flags.insert(control::Flag::I, (value & (0x01 << I_BIT)) != 0);
  }
}

impl bus::Device<control::FlagsRegister> for FlagsRegister {
  fn update(&mut self, control: control::FlagsRegister) -> Result<(), Error> {
    match control {
      control::FlagsRegister { Data: control::ReadWrite::Read, C: Some(_), I: _, .. } |
      control::FlagsRegister { Data: control::ReadWrite::Read, C: _, I: Some(_), .. } =>
        Err(Error::UpdateConflict(vec![
          String::from("FlagsRegister:Data"),
          String::from("FlagsRegister:Set"),
        ])),
      control => {
        self.control = control;
        Ok(())
      },
    }
  }

  fn read(&self) -> Result<bus::State, Error> {
    Ok(bus::State {
      data: if let control::ReadWrite::Write = self.control.Data {
        Some(self.to_value())
      } else {
        None
      },
      addr: None,
    })
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    if let control::ReadWrite::Read = self.control.Data {
      self.from_value(state.read_data()?);
    }

    if let Some(value) = self.control.C {
      self.flags.insert(control::Flag::C, value);
    }

    if let Some(value) = self.control.I {
      self.flags.insert(control::Flag::I, value);
    }

    Ok(())
  }
}

impl fmt::Display for FlagsRegister {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "FlagsRegister({:#X})", self.to_value())
  }
}

impl fmt::Debug for FlagsRegister {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "FlagsRegister({:#X} D={:?} C={:?} I={:?})", self.to_value(), self.control.Data, self.control.C, self.control.I)
  }
}
