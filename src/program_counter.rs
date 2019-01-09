
use std::fmt;

use crate::bus;
use crate::control;
use crate::error::Error;


fn to_bytes(value: u16) -> [u8; 2] {
  [(value >> 8) as u8, value as u8]
}
fn from_bytes(value: &[u8; 2]) -> u16 {
  ((value[0] << 8) as u16) | (value[1] as u16)
}

#[derive(PartialEq, Eq)]
pub struct ProgramCounter {
  control: control::ProgramCounter,
  value: [u8; 2],
}

impl ProgramCounter {
  pub fn new() -> ProgramCounter {
    ProgramCounter {
      control: control::ProgramCounter::new(),
      value: [0x00, 0x00],
    }
  }
}

impl bus::Device<control::ProgramCounter> for ProgramCounter {
  fn update(&mut self, control: control::ProgramCounter) -> Result<(), Error> {
    use crate::control::ReadWrite;
    use crate::control::ProgramCounterCount as CountControl;

    #[allow(non_snake_case)]
    let control::ProgramCounter { DataL, DataH, Count, Addr } = control;
    if DataH == ReadWrite::Write && DataL == ReadWrite::Write {
      Err(Error::BusConflict(vec![
        String::from("ProgramCounter:H"),
        String::from("ProgramCounter:L"),
      ]))
    } else if Addr == ReadWrite::Read && DataH == ReadWrite::Read {
      Err(Error::UpdateConflict(vec![
        String::from("ProgramCounter:HL"),
        String::from("ProgramCounter:H"),
      ]))
    } else if Addr == ReadWrite::Read && DataL == ReadWrite::Read {
      Err(Error::UpdateConflict(vec![
        String::from("ProgramCounter:HL"),
        String::from("ProgramCounter:L"),
      ]))
    } else if Addr == ReadWrite::Read && Count != CountControl::None {
      Err(Error::UpdateConflict(vec![
        String::from("ProgramCounter:HL"),
        String::from("ProgramCounter:Count"),
      ]))
    } else if DataH == ReadWrite::Read && Count != CountControl::None {
      Err(Error::UpdateConflict(vec![
        String::from("ProgramCounter:H"),
        String::from("ProgramCounter:Count"),
      ]))
    } else if DataL == ReadWrite::Read && Count == CountControl::Increment {
      Err(Error::UpdateConflict(vec![
        String::from("ProgramCounter:L"),
        String::from("ProgramCounter:Count"),
      ]))
    } else {
      self.control = control::ProgramCounter {
        Count: Count,
        DataL: DataL,
        DataH: DataH,
        Addr: Addr,
      };
      Ok(())
    }
  }

  fn read(&self) -> bus::State {
    bus::State {
      data: if let control::ReadWrite::Write = self.control.DataH {
        Some(self.value[0])
      } else if let control::ReadWrite::Write = self.control.DataL {
        Some(self.value[1])
      } else {
        None
      },
      addr: if let control::ReadWrite::Write = self.control.Addr {
        Some(from_bytes(&self.value))
      } else {
        None
      },
    }
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    use crate::control::ProgramCounterCount as CountControl;

    if let control::ReadWrite::Read = self.control.DataH {
      self.value[0] = state.read_data()?;
    }
    if let control::ReadWrite::Read = self.control.DataL {
      self.value[1] = state.read_data()?;
    }
    if let control::ReadWrite::Read = self.control.Addr {
      self.value = to_bytes(state.read_addr()?);
    }

    match self.control.Count {
      CountControl::Increment => {
        if self.value[1] == 0xFF {
          self.value = [self.value[0] + 1, 0x00];
        } else {
          self.value[1] += 1;
        }
      },
      CountControl::Carry => self.value[0] += 1,
      CountControl::Borrow => self.value[0] -= 1,
      CountControl::None => (),
    }

    Ok(())
  }
}

impl fmt::Display for ProgramCounter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "ProgramCounter({:#X})", from_bytes(&self.value))
  }
}

impl fmt::Debug for ProgramCounter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "ProgramCounter({:#X} D=({:?},{:?}) A={:?} C={:?})",
      from_bytes(&self.value), self.control.DataH, self.control.DataL, self.control.Addr, self.control.Count)
  }
}
