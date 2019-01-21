
pub mod set;

use std::fmt;

use crate::control::{Control, Flag};


pub trait Set {
	fn start(&self) -> u16;
  fn fetch(&self) -> Micro;
  fn get(&self, op: u8) -> Micro;
  fn interrupt(&self) -> Micro;
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Micro {
  Code(Vec<Control>),
  Compress(Vec<Control>),
  Branch(Flag, Box<Micro>, Box<Micro>),
}

impl fmt::Display for Micro {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}
