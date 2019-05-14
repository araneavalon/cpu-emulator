
mod microcode;
mod instructions;
mod control;

use std::fmt;
use crate::error::{Error, Result};
use super::components::Flags;
use self::microcode::MicrocodeArray;
use self::instructions::{
  Instructions,
  Iter,
};

pub use self::control::*;


#[derive(Debug)]
enum State {
  Init,
  Fetch,
  Run(Iter),
}

pub struct ControlLogic {
  microcode: MicrocodeArray,
  instructions: Instructions,
  previous: Control,
  state: State,
  cycle: usize,
  fetch: usize,
}

impl ControlLogic {
  pub fn new() -> Result<ControlLogic> {
    let microcode = self::microcode::array();
    Ok(ControlLogic {
      microcode,
      instructions: Instructions::new(&microcode)?,
      previous: Control::new(),
      state: State::Init,
      cycle: 0,
      fetch: 0,
    })
  }

  pub fn decode(&mut self, op: u16, flags: &Flags) -> Result<Control> {
    match self.state {
      State::Fetch => {
        let instruction = self.instructions.decode(&self.microcode, op)?;
        self.state = State::Run(instruction);
      },
      State::Init => {
        self.state = State::Run(self.instructions.init());
      },
      State::Run(_) => (),
    }

    let c = match &mut self.state {
      State::Fetch => return Err(Error::Impossible(op, "It should be literally impossible for ControlLogic.state to be Fetch.")),
      State::Init => return Err(Error::Impossible(op, "It should be literally impossible for ControlLogic.state to be Init.")),
      State::Run(instruction) => {
        match instruction.next() {
          Some(c) if flags.test(c.branch.negate, c.branch.condition) => c,
          _ => {
            self.state = State::Fetch;
            self.fetch += 1;
            self.instructions.fetch()
          },
        }
      },
    };

    let out = c.previous(self.previous);
    self.previous = c;
    self.cycle += 1;
    Ok(out)
  }
}

impl fmt::Debug for ControlLogic {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "ControlLogic(fetch:={}, cycle:={})", self.fetch, self.cycle)
  }
}

impl fmt::Display for ControlLogic {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self.state)
  }
}
