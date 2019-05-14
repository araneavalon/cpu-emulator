
use nom::types::CompleteStr;
use std::error;
use std::fmt;
use std::io;


#[macro_export]
macro_rules! try_kind {
  ( *$line:ident, $e:expr ) => {{
    let line = *$line;
    try_kind!(line, $e)
  }};
  ( $line:ident, $e:expr ) => {{
    use $crate::error::Error;
    match $e {
      Err(kind) => return Err(Error::new($line, kind)),
      Ok(value) => value,
    }
  }};
}


pub type KindResult<T> = std::result::Result<T, ErrorKind>;
pub type Result<T> = std::result::Result<T, Error>;


#[derive(Debug)]
pub enum ErrorKind {
  InvalidAddress(u16, u16),
  UnknownLabel(String),
  UnknownRelative(String, usize),
  DuplicateLabel(String),
  OutOfRange(u16, u16, i32, i32),
  StoreConstant,
  Impossible(&'static str),
  InvalidFilename(String),
  Parser(String),
  File(io::Error),
}

impl fmt::Display for ErrorKind {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ErrorKind::InvalidAddress(min, max)      => write!(f, "InvalidAddress([{}, {}])", min, max),
      ErrorKind::UnknownLabel(label)           => write!(f, "UnknownLabel({})", label),
      ErrorKind::UnknownRelative(label, index) => write!(f, "UnknownRelative({}, {})", label, index),
      ErrorKind::DuplicateLabel(label)         => write!(f, "DuplicateLabel({})", label),
      ErrorKind::OutOfRange(v_min, v_max, min, max) =>
        write!(f, "OutOfRange(value:=[{}, {}], range:=[{}, {}])", v_min, v_max, min, max),
      ErrorKind::StoreConstant                 => write!(f, "StoreConstant"),
      ErrorKind::Impossible(message)           => write!(f, "Impossible({})", message),
      ErrorKind::InvalidFilename(file)         => write!(f, "InvalidFilename(\"{}\")", file),
      ErrorKind::Parser(message)               => write!(f, "Parser({})", message),
      ErrorKind::File(error)                   => write!(f, "File({})", error),
    }
  }
}

#[derive(Debug)]
pub struct Error {
  file: Option<String>,
  line: Option<usize>,
  kind: ErrorKind,
}

impl Error {
  pub fn new(line: usize, kind: ErrorKind) -> Error {
    Error { file: None, line: Some(line), kind }
  }

  pub fn parser<'a>(index: usize, error: nom::Err<CompleteStr<'a>>) -> Error {
    Error { file: None, line: Some(index + 1), kind: ErrorKind::Parser(format!("{}", error)) }
  }

  pub fn invalid_filename(file: String, bad_file: String) -> Error {
    Error { file: Some(file), line: None, kind: ErrorKind::InvalidFilename(bad_file) }
  }

  pub fn file(file: String, error: io::Error) -> Error {
    Error { file: Some(file), line: None, kind: ErrorKind::File(error) }
  }
}

impl error::Error for Error {}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match (&self.file, &self.line) {
      (Some(file), Some(line)) => write!(f, "{}:{:6}: {}", file, line, self.kind),
      (Some(file), None) => write!(f, "{}: {}", file, self.kind),
      (None, Some(line)) => write!(f, "[UNKNOWN]:{:6}: {}", line, self.kind),
      (None, None) => write!(f, "{}", self.kind),
    }
  }
}

impl From<io::Error> for Error {
  fn from(error: io::Error) -> Error {
    Error { file: None, line: None, kind: ErrorKind::File(error) }
  }
}
