
use std::error;
use std::fmt;
use std::io;


pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
  DataBusConflict(u16, &'static str),
  DataBusUnused(u16),
  AddressBusConflict(u16, &'static str),
  AddressBusUnused(u16),
  InvalidExtraRegister(u16, u16),
  InvalidRegister(u16, u16, u16),
  InvalidBinaryOp(u16, u16),
  InvalidUnaryOp(u16, u16),
  InvalidCondition(u16, u16),
  InvalidRead(u16, &'static str),
  InvalidWrite(u16, &'static str),
  InvalidInterrupt(u16),
  Impossible(u16, &'static str),
  ParseFloatError(std::num::ParseFloatError),
  Sdl2StringError(String),
  Sdl2WindowError(sdl2::video::WindowBuildError),
  Sdl2IntegerError(sdl2::IntegerOrSdlError),
  File(String, io::Error),
  Assembler(String, assembler::Error),
  InvalidROM,
}

impl error::Error for Error {}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::DataBusConflict(op, name) =>
        write!(f, "DataBusConflict(0x{:04X}): {} caused a conflict on the data bus.", op, name),
      Error::DataBusUnused(op) =>
        write!(f, "DataBusUnused(0x{:04X}): The data bus went unused this cycle. Check microcode for optimizations?", op),
      Error::AddressBusConflict(op, name) =>
        write!(f, "AddressBusConflict(0x{:04X}): {} caused a conflict on the address bus.", op, name),
      Error::AddressBusUnused(op) =>
        write!(f, "AddressBusUnused(0x{:04X}): The address bus went unused this cycle. Address bus should always be set.", op),
      Error::InvalidExtraRegister(op, value) =>
        write!(f, "InvalidExtraRegister(0x{:04X}): 0b{:02b} is not a valid Extra Register.", op, value),
      Error::InvalidRegister(op, offset, value) =>
        write!(f, "InvalidRegister(0x{:04X}, offset:={}): 0b{:03b} is not a valid Register.", op, offset, value),
      Error::InvalidBinaryOp(op, value) =>
        write!(f, "InvalidBinaryOp(0x{:04X}): 0b{:03b} is not a valid Binary Alu Operation.", op, value),
      Error::InvalidUnaryOp(op, value) =>
        write!(f, "InvalidUnaryOp(0x{:04X}): 0b{:03b} is not a valid Unary Alu Operation.", op, value),
      Error::InvalidCondition(op, value) =>
        write!(f, "InvalidCondition(0x{:04X}): 0b{:03b} is not a valid Condition.", op, value),
      Error::InvalidRead(op, message) => 
        write!(f, "InvalidRead(0x{:04X}): {}", op, message),
      Error::InvalidWrite(op, message) =>
        write!(f, "InvalidWrite(0x{:04X}): {}", op, message),
      Error::InvalidInterrupt(interrupt) =>
        write!(f, "InvalidInterrupt({}): Hardware Interrupts must be in the range [0,7].", interrupt),
      Error::Impossible(op, message) =>
        write!(f, "Impossible(0x{:04X}): {}", op, message),
      Error::ParseFloatError(error) =>
        write!(f, "ParseFloatError: {}", error),
      Error::Sdl2StringError(error) =>
        write!(f, "Sdl2: {}", error),
      Error::Sdl2WindowError(error) =>
        write!(f, "Sdl2: {}", error),
      Error::Sdl2IntegerError(error) =>
        write!(f, "Sdl2: {}", error),
      Error::File(path, error) =>
        write!(f, "File({}): {}", path, error),
      Error::Assembler(path, error) =>
        write!(f, "Assembler({}): {}", path, error),
      Error::InvalidROM =>
        write!(f, "InvalidROM: No ROM file provided."),
    }
  }
}

impl From<std::num::ParseFloatError> for Error {
  fn from(error: std::num::ParseFloatError) -> Error {
    Error::ParseFloatError(error)
  }
}

impl From<sdl2::video::WindowBuildError> for Error {
  fn from(error: sdl2::video::WindowBuildError) -> Error {
    Error::Sdl2WindowError(error)
  }
}

impl From<sdl2::IntegerOrSdlError> for Error {
  fn from(error: sdl2::IntegerOrSdlError) -> Error {
    Error::Sdl2IntegerError(error)
  }
}

impl From<Error> for fmt::Error {
  fn from(_: Error) -> fmt::Error {
    fmt::Error
  }
}

#[macro_export]
macro_rules! sdl_e {
  (__String, $e:expr ) => {
    match $e {
      Err(message) => Err(Error::Sdl2StringError(message)),
      Ok(value) => Ok(value),
    }
  };
  ( sdl2::init() ) => {
    sdl_e!(__String, sdl2::init())
  };
  ( $e:ident.video() ) => {
    sdl_e!(__String, $e.video())
  };
  ( $e:ident.event_pump() ) => {
    sdl_e!(__String, $e.event_pump())
  };
  ( $e:ident.set_scale($h:expr, $v:expr) ) => {
    sdl_e!(__String, $e.set_scale($h, $v))
  };
  ( $e:ident.draw_point(($x:expr, $y:expr)) ) => {
    sdl_e!(__String, $e.draw_point(($x, $y)))
  };
  ( $e:expr ) => {
    $e
  };
}
