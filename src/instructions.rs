
mod first;

use crate::control::{
  Control,
  ReadWrite,
  Flag,
};


pub type Set {
  fn fetch() -> Vec<Micro> {
    let c = Control::new();
    c.ProgramCounter.Addr = ReadWrite::Write;
    c.Memory.Data = ReadWrite::Read;
    c.Instruction.Data = ReadWrite::Read;
    vec![Micro::Static(c)]
  }

  fn instruction(op: u8) -> Vec<Micro>;
}


#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Micro {
  Static(Control),
  Flag(Flag, Control, Control),
}
