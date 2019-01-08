
use std::error::Error as StdError;
use std::fmt;


#[derive(Debug)]
pub enum Error {
  AmbiguousBus(usize),
  AmbiguousRead(Vec<String>),
  AmbiguousWrite(Vec<String>),
}

impl StdError for Error {
  fn description(&self) -> &str {
    match self {
      Error::AmbiguousBus(_) => "Bus read resulted in ambiguous value.",
      Error::AmbiguousRead(_) => "Attempted to read value from multiple locations.",
      Error::AmbiguousWrite(_) => "Attempted to write value from multiple locations.",
    }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::AmbiguousBus(count) => write!(f, "Unable to read bus, value would be ambiguous due to {} drivers.", count),
      Error::AmbiguousRead(sources) => write!(f, "Attempted to read value from multiple locations: {:?}.", sources),
      Error::AmbiguousWrite(sources) => write!(f, "Attempted to write value from multiple locations: {:?}.", sources),
    }
  }
}
