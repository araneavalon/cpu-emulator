
mod microcode;
mod instructions;
mod control;
mod stack_sequence;

use std::fmt;

use crate::components::flags::Flags;
use crate::control::instructions::Instructions;
use crate::control::stack_sequence::StackSequence;

pub use crate::control::control::*;


pub struct ControlLogic {
  cycle: usize,
  stack: Option<StackSequence>,
  previous: Control,

  instructions: Instructions,
}

impl ControlLogic {
  pub fn new() -> ControlLogic {
    ControlLogic {
      cycle: 0,
      stack: None,
      previous: Control::new(),

      instructions: Instructions::new(),
    }
  }

  fn fetch(&mut self) -> Control {
    self.cycle = 0;
    self.stack = None;
    self.instructions.fetch()
  }

  fn stack_sequence(&mut self) -> Control {
    if let Some(seq) = &mut self.stack {
      if let Some(c) = seq.next() {
        return c
      }
    }
    self.fetch()
  }

  fn _decode(&mut self, op: u16, flags: &Flags) -> Control {
    if let Some(_seq) = &self.stack {
      return self.stack_sequence();
    }

    let ins = self.instructions.get(op);
    if self.cycle >= ins.len() {
      return self.fetch();
    }

    let c = ins[self.cycle].decode(op);
    if c.stack_sequence {
      self.stack = Some(StackSequence::new(op, c));
      return self.stack_sequence();
    }

    if flags.test(c.branch.negate, c.branch.condition) {
      c
    } else {
      self.fetch()
    }
  }

  pub fn decode(&mut self, op: u16, flags: &Flags) -> Control {
    let c = self._decode(op, flags);
    let out = c.previous(self.previous);
    self.cycle += 1;
    self.previous = c;
    out
  }
}

impl fmt::Debug for ControlLogic {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "ControlLogic {{ {:?}, {:?}, {:?} }}", self.cycle, self.stack, self.previous)
  }
}
