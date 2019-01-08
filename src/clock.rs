
use crate::error::Error;


pub trait Clock {
  fn clock(&self) -> Result<(), Error>;
}
