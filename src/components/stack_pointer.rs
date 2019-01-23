
use std::fmt;

use crate::bus;
use crate::control;
use crate::error::Error;


const MIN: u16 = 0x0400;
const MAX: u16 = 0x07FF;

#[derive(PartialEq, Eq)]
pub struct StackPointer {
  control: control::StackPointer,
  value: u16,
}

impl StackPointer {
  pub fn new() -> StackPointer {
    StackPointer {
      control: control::StackPointer::new(),
      value: MAX,
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

  fn read(&self) -> Result<bus::State, Error> {
    Ok(bus::State {
      data: None,
      addr: if let control::Write::Write = self.control.Addr {
        Some(self.value)
      } else {
        None
      },
    })
  }

  fn clk(&mut self, _state: &bus::State) -> Result<(), Error> {
    if self.control.Reset {
      self.value = MAX;
    }
    Ok(())
  }
}

impl fmt::Display for StackPointer {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "0x{:04X}", self.value)
  }
}

impl fmt::Debug for StackPointer {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "0x{:04X} (Address={:?}, IncDec={:?}) [StackPointer]", self.value, self.control.Addr, self.control.Count)
  }
}
