
pub mod set;

use crate::control::{Control, Flag};


pub trait Set {
	fn init(&self) -> Vec<Control>;
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
