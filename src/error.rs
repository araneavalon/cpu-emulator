
use std::error::Error as StdError;
use std::fmt;


#[derive(Debug)]
pub enum Error {
  InvalidRead(String),
  BusConflict(Vec<String>),
  UpdateConflict(Vec<String>),
}

impl StdError for Error {
  fn description(&self) -> &str {
    match self {
      Error::InvalidRead(_) => "Bus read resulted in invalid value.",
      Error::BusConflict(_) => "Attempted to write multiple devices to bus.",
      Error::UpdateConflict(_) => "Attempted to write multiple sources to device.",
    }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::InvalidRead(bus) => write!(f, "Unable to read from bus {}, value is invalid.", bus),
      Error::BusConflict(sources) => write!(f, "Unable to write multiple devices to bus: {:?}.", sources),
      Error::UpdateConflict(sources) => write!(f, "Unable to update device from multiple sources: {:?}.", sources),
    }
  }
}
