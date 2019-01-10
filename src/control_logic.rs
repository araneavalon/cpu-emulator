
use std::fmt;

use crate::bus;
use crate::control;
use crate::error::Error;


#[derive(PartialEq, Eq)]
pub struct ControlLogic {
  control: control::Instruction,
  cycle: u8,
}

impl ControlLogic {
  pub fn new() -> ControlLogic {
    ControlLogic {
      control: control::Instruction::new(),
      cycle: u8,
    }
  }

  pub fn get_control(&self) -> control::Control {
    control::Control::new()
  }
}

impl bus::Device<control::Instruction> for ControlLogic {
  fn update(&mut self, control: control::Instruction) -> Result<(), Error> {
    self.control = control;
    Ok(())
  }

  fn read(&self) -> bus::State {
    bus::State {
      data: None,
      addr: None,
      // addr: if let control::Write::Write = self.control.Addr {
      //   Some(bus::Addr::Full(self.value))
      // } else {
      //   None
      // },
    }
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    // use crate::control::StackPointerCount as CountControl;

    // match self.control.Count {
    //   CountControl::Increment => {
    //     if self.value == 0x01 {
    //       self.value = 0x01
    //     } else {
    //       self.value += 1
    //     }
    //   },
    //   CountControl::Decrement => {
    //     if self.value == 0x01 {
    //       self.value = 0x01
    //     } else {
    //       self.value -= 1
    //     }
    //   },
    //   CountControl::None => (),
    // }

    Ok(())
  }
}

impl fmt::Display for ControlLogic {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "ControlLogic()")
  }
}

impl fmt::Debug for ControlLogic {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "ControlLogic()")
  }
}
