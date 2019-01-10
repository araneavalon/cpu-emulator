
use std::fmt;

use crate::bus;
use crate::control;
use crate::error::Error;


#[derive(PartialEq, Eq)]
pub struct HRegister {
  control: control::HRegister,
  buffer: u8,
  value: u8
}

impl HRegister {
  pub fn new() -> HRegister {
    HRegister {
      control: control::HRegister::new(),
      buffer: 0x00,
      value: 0x00,
    }
  }
}

impl bus::Device<control::HRegister> for HRegister {
  fn update(&mut self, control: control::HRegister) -> Result<(), Error> {
    if control.Latch == control::Write::Write && control.Count != control::IncDec::None {
      Err(Error::UpdateConflict(vec![
        String::from("HRegister:Latch"),
        String::from("HRegister:Count"),
      ]))
    } else {
      self.control = control;

      if let control::Write::Write = self.control.Latch {
        self.value = self.buffer;
      }

      if let control::IncDec::Increment = self.control.Count {
        if self.value == 0xFF {
          self.value = 0x00;
        } else {
          self.value += 1;
        }
      } else if let control::IncDec::Increment = self.control.Count {
        if self.value == 0x00 {
          self.value = 0xFF;
        } else {
          self.value -= 1;
        }
      }

      Ok(())
    }
  }

  fn read(&self) -> bus::State {
    bus::State {
      data: None,
      addr: if let control::Write::Write = self.control.Addr {
        Some(bus::Addr::High(self.value))
      } else {
        None
      },
    }
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    if let control::Read::Read = self.control.Data {
      self.buffer = state.read_data()?;
    }
    Ok(())
  }
}

impl fmt::Display for HRegister {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "HRegister(H={:#X} H'={:#X})", self.value, self.buffer)
  }
}

impl fmt::Debug for HRegister {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "HRegister(H={:#X} H'={:#X} D={:?} L={:?} C={:?} A={:?})",
      self.value, self.buffer, self.control.Data, self.control.Latch, self.control.Count, self.control.Addr)
  }
}
