
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use crate::bus::Bus;
use crate::clock::Clock;
use crate::error::Error;


#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Control {
  ReadI,
  WriteR,
  WriteD,
}


#[derive(Debug, PartialEq, Eq)]
pub struct Register {
  control: RefCell<HashMap<Control, bool>>,
  value: RefCell<u8>,
  i: Rc<Bus>,
}

impl Register {
  pub fn new(i: Rc<Bus>) -> Register {
    Register {
      control: RefCell::new(hash_map!{
        Control::ReadI => false,
        Control::WriteR => false,
        Control::WriteD => false,
      }),
      value: RefCell::new(0),
      i: i,
    }
  }

  pub fn set(&self, control: Control, value: bool) {
    self.control.borrow_mut().insert(control, value);
  }

  pub fn read(&self, control: &Control) -> Result<Option<u8>, Error> {
    if self.control.borrow()[control] {
      Ok(Some(*self.value.borrow()))
    } else {
      Ok(None)
    }
  }
}

impl Clock for Register {
  fn clock(&self) -> Result<(), Error> {
    if self.control.borrow()[&Control::ReadI] {
      *self.value.borrow_mut() = self.i.read()?;
    }
    Ok(())
  }
}

impl fmt::Display for Register {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "0x{:X}", *self.value.borrow())
  }
}
