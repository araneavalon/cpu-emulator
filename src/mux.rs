
use std::rc::Rc;
use std::cell::RefCell;

use crate::error::Error;
use crate::bus::Bus;


#[derive(Debug, PartialEq, Eq)]
pub struct Mux {
  select: RefCell<Option<usize>>,
  buses: Vec<Rc<Bus>>,
}

impl Mux {
  pub fn new(buses: Vec<Rc<Bus>>) -> Mux {
    Mux {
      select: RefCell::new(None),
      buses: buses,
    }
  }

  pub fn set(&self, value: Option<usize>) {
    *self.select.borrow_mut() = match value {
      Some(value) => Some(value % self.buses.len()),
      None => None,
    };
  }

  pub fn read(&self) -> Result<Option<u8>, Error> {
    match *self.select.borrow() {
      Some(value) => Ok(Some(self.buses[value].read()?)),
      None => Ok(None),
    }
  }
}
