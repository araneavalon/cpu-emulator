
use std::cell::RefCell;

use crate::error::Error;


#[derive(Debug, PartialEq, Eq)]
pub struct Input {
  value: RefCell<Option<u8>>,
}

impl Input {
  pub fn new() -> Input {
    Input {
      value: RefCell::new(None),
    }
  }

  pub fn set(&self, value: Option<u8>) {
    *self.value.borrow_mut() = value;
  }

  pub fn read(&self) -> Result<Option<u8>, Error> {
    Ok(*self.value.borrow())
  }
}
