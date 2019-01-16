
use std::fmt;

use crate::math::*;
use crate::bus;
use crate::control;
use crate::error::Error;


#[derive(Debug, PartialEq, Eq)]
pub struct ProgramCounter {
  control: control::ProgramCounter,
  value: u16,
}

impl ProgramCounter {
  pub fn new() -> ProgramCounter {
    ProgramCounter {
      control: control::ProgramCounter::new(),
      value: 0x0000,
    }
  }
}

impl bus::Device<control::ProgramCounter> for ProgramCounter {
  fn update(&mut self, control: control::ProgramCounter) -> Result<(), Error> {
    use crate::control::ReadWrite;
    use crate::control::Write;
    use crate::control::IncDec;

    #[allow(non_snake_case)]
    let control::ProgramCounter { DataL, DataH, Count, Addr } = control;
    if Count == IncDec::Decrement {
      panic!("Unsupported operation for ProgramCounter: Decrement");
    } else if DataH == Write::Write && DataL == Write::Write {
      Err(Error::BusConflict(vec![
        String::from("ProgramCounter:H"),
        String::from("ProgramCounter:L"),
      ]))
    } else {
      self.control = control::ProgramCounter {
        Count: Count,
        DataL: DataL,
        DataH: DataH,
        Addr: Addr,
      };

      if let IncDec::Increment = self.control.Count {
        if self.value == 0xFFFF {
          self.value = 0x0000;
        } else {
          self.value += 1;
        }
      }

      Ok(())
    }
  }

  fn read(&self) -> Result<bus::State, Error> {
    Ok(bus::State {
      data: if let control::Write::Write = self.control.DataH {
        Some(to_bytes(self.value)[0])
      } else if let control::Write::Write = self.control.DataL {
        Some(to_bytes(self.value)[1])
      } else {
        None
      },
      addr: if let control::ReadWrite::Write = self.control.Addr {
        Some(self.value)
      } else {
        None
      },
    })
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    if let control::ReadWrite::Read = self.control.Addr {
      self.value = state.read_addr()?;
    }
    Ok(())
  }
}

impl fmt::Display for ProgramCounter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "0x{:04X} (Address={}, DataH={}, DataL={}, IncDec={}) [ProgramCounter]",
      self.value, self.control.Addr, self.control.DataH, self.control.DataL, self.control.Count)
  }
}
