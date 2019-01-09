
use std::fmt;

use crate::bus;
use crate::control;
use crate::error::Error;


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
    Ok(())
  }

  fn read(&self) -> bus::State {
    bus::State {
      data: None,
      addr: if let control::Write::Write = self.control.Addr {
        Some(self.value)
      } else {
        None
      },
    }
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    use crate::control::StackPointerCount as CountControl;

    match self.control.Count {
      CountControl::Increment => {
        if self.value == MAX {
          self.value = MIN
        } else {
          self.value += 1
        }
      },
      CountControl::Decrement => {
        if self.value == MIN {
          self.value = MAX
        } else {
          self.value -= 1
        }
      },
      CountControl::None => (),
    }

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
