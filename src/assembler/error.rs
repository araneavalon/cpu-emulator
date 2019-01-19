
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
pub enum Type {
  Word,
  Byte,
}


#[derive(Debug)]
pub enum Error {
  ParseInt(std::num::ParseIntError),

  InvalidOperation(Source, Reason),
  UnknownName(String),
  InvalidType(Type, Type),
  SectionOverlap(u16, u16),

  IncompleteParse(String),
}

impl error::Error for Error {
  fn description(&self) -> &str {
    match self {
      Error::ParseInt(error) => error.description(),
      Error::InvalidOperation(_, _) => "Attempted to assemble invalid operation.",
      Error::UnknownName(_) => "Attempted to access a name that does not exist.",
      Error::InvalidType(_, _) => "Expression resolved to invalid type.",
      Error::SectionOverlap(_, _) => "Sections can not overlap.",
      Error::IncompleteParse(_) => "Parser failed to consume all input.",
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
      Error::UnknownName(name) => write!(f, "UnknownName: \"{}\"", name),
      Error::InvalidType(expected, real) => write!(f, "InvalidType: Wanted {:?}, got {:?}.", expected, real),
      Error::SectionOverlap(end, start) => write!(f, "Section beginning at \"0x{:04x}\" overlaps ending at \"0x{:04x}\".", start, end),
      Error::IncompleteParse(remaining) => write!(f, "Incomplete Parse: \"{}\"", remaining),
    }
  }
}


impl From<std::num::ParseIntError> for Error {
  fn from(error: std::num::ParseIntError) -> Error {
    Error::ParseInt(error)
  }
}
