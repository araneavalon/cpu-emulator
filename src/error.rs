
use std::error::Error as StdError;
use std::fmt;


#[derive(Debug)]
pub enum Error {
  Error(String),
  InvalidRead(String),
  InvalidWrite(String),
  BusConflict(Vec<String>),
  UpdateConflict(Vec<String>),
}

impl StdError for Error {
  fn description(&self) -> &str {
    match self {
      Error::Error(_) => "Error",
      Error::InvalidRead(_) => "Bus read resulted in invalid value.",
      Error::InvalidWrite(_) => "Attempted to write to an invalid location.",
      Error::BusConflict(_) => "Attempted to write multiple devices to bus.",
      Error::UpdateConflict(_) => "Attempted to write multiple sources to device.",
    }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::Error(message) => write!(f, "Error: {}", message),
      Error::InvalidRead(bus) => write!(f, "Unable to read from bus \"{}\", value is invalid.", bus),
      Error::InvalidWrite(message) => write!(f, "Invalid Write: {}", message),
      Error::BusConflict(sources) => write!(f, "Unable to write multiple devices to bus: {:?}.", sources),
      Error::UpdateConflict(sources) => write!(f, "Unable to update device from multiple sources: {:?}.", sources),
    }
  }
}
