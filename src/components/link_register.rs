
use std::fmt;
use crate::control::Control;
use crate::error::Result;
use super::BusComponent;


#[derive(Debug)]
pub struct LinkRegister {
  control: Control,
  value: u16,
}

impl LinkRegister {
  pub fn new() -> LinkRegister {
    LinkRegister {
      control: Control::new(),
      value: 0x0000,
    }
  }

  pub fn link(&mut self, value: u16) {
    if self.control.link {
      self.value = value;
    }
  }
}

impl fmt::Display for LinkRegister {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, " LR ")?;
    if self.control.lr.load && self.control.lr.out {
      write!(f, " <>")?;
    } else if self.control.lr.load {
      write!(f, " <=")?;
    } else if self.control.lr.out {
      write!(f, " =>")?;
    } else {
      write!(f, " ==")?;
    }
    write!(f, " 0x{:04X}", self.value)?;
    if self.control.lr.increment {
      write!(f, " ++")?;
    }
    Ok(())
  }
}

impl BusComponent for LinkRegister {
  fn name(&self) -> &'static str {
    "LinkRegister"
  }

  fn set_control(&mut self, control: Control) {
    self.control = control;

    if self.control.lr.increment {
      self.value = self.value.wrapping_add(1);
    }
  }

  fn load(&mut self, value: u16) -> Result<()> {
    if self.control.lr.load {
      self.value = value;
    }
    Ok(())
  }

  fn data(&self) -> Result<Option<u16>> {
    if self.control.lr.out {
      Ok(Some(self.value))
    } else {
      Ok(None)
    }
  }
}
