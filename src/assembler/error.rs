
use std::error;
use std::fmt;

use crate::assembler::tokens::{Address, Register, Op};


#[derive(Debug)]
pub enum Source {
  Register(Register),
  Address(Address),
  Operation(Op),
}

#[derive(Debug)]
pub enum Reason {
  Disabled,
  Invalid,
}


#[derive(Debug)]
pub enum Error {
  ParseInt(std::num::ParseIntError),

  InvalidOperation(Source, Reason),
  UnknownLabel(String),
  SectionOverlap(u16, u16),
}

impl error::Error for Error {
  fn description(&self) -> &str {
    match self {
      Error::ParseInt(error) => error.description(),
      Error::InvalidOperation(_, _) => "Attempted to assemble invalid operation.",
      Error::UnknownLabel(_) => "Attempted to access a label that does not exist.",
      Error::SectionOverlap(_, _) => "Sections can not overlap.",
    }
  }

  fn cause(&self) -> Option<&error::Error> {
    match self {
      Error::ParseInt(error) => Some(error),
      _ => None,
    }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::ParseInt(error) => write!(f, "ParseIntError: {}", error),
      Error::InvalidOperation(source, reason) => write!(f, "InvalidOperation: {:?} is {:?}", source, reason),
      Error::UnknownLabel(label) => write!(f, "UnknownLabel: \"{}\"", label),
      Error::SectionOverlap(end, start) => write!(f, "Section beginning at \"0x{:04x}\" overlaps ending at \"0x{:04x}\".", start, end),
    }
  }
}


impl From<std::num::ParseIntError> for Error {
  fn from(error: std::num::ParseIntError) -> Error {
    Error::ParseInt(error)
  }
}
