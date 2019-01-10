
use std::fmt;

use crate::bus;
use crate::control;
use crate::error::Error;


const MIN: u16 = 0x0C00;
const MAX: u16 = 0x0FFF;

#[derive(PartialEq, Eq)]
pub struct StackPointer {
  control: control::StackPointer,
  value: u16,
}

impl StackPointer {
  pub fn new() -> StackPointer {
    StackPointer {
      control: control::StackPointer::new(),
      value: MIN, // 10bit, actual range is 0x0C00 to 0x0FFF
    }
  }
}

impl bus::Device<control::StackPointer> for StackPointer {
  fn update(&mut self, control: control::StackPointer) -> Result<(), Error> {
    self.control = control;

    match self.control.Count {
      control::IncDec::Increment => {
        if self.value == MAX {
          self.value = MIN
        } else {
          self.value += 1
        }
      },
      control::IncDec::Decrement => {
        if self.value == MIN {
          self.value = MAX
        } else {
          self.value -= 1
        }
      },
      control::IncDec::None => (),
    }

    Ok(())
  }

  fn read(&self) -> bus::State {
    bus::State {
      data: None,
      addr: if let control::Write::Write = self.control.Addr {
        Some(bus::Addr::Full(self.value))
      } else {
        None
      },
    }
  }

  fn clk(&mut self, _state: &bus::State) -> Result<(), Error> {
    Ok(())
  }
}

impl fmt::Display for StackPointer {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "StackPointer({:#X})", self.value)
  }
}

impl fmt::Debug for StackPointer {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "StackPointer({:#X} A={:?} C={:?})", self.value, self.control.Addr, self.control.Count)
  }
}
