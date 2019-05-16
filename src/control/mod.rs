
mod microcode;
mod instructions;
mod control;

use std::fmt;
use crate::error::{Error, Result};
use super::components::Flags;
use super::components::InstructionRegister;
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
  interrupt: Option<u16>,
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
      interrupt: None,
      cycle: 0,
      fetch: 0,
    })
  }

  pub fn interrupt(&mut self, interrupt: u16) -> Result<()> {
    if interrupt > 7 {
      Err(Error::InvalidInterrupt(interrupt))
    } else {
      self.interrupt = Some(interrupt);
      Ok(())
    }
  }

  pub fn decode(&mut self, op: u16, flags: &Flags, ir: &mut InstructionRegister) -> Result<Control> {
    match &mut self.state {
      State::Fetch => {
        let instruction = self.instructions.decode(&self.microcode, op)?;
        self.state = State::Run(instruction);
      },
      State::Init => {
        self.state = State::Run(self.instructions.init());
      },
      State::Run(instruction) => {
        if let None = instruction.peek() {
          if let Some(i) = self.interrupt {
            let (op, instruction) = self.instructions.interrupt(&self.microcode, i)?;
            self.interrupt = None;
            ir.set(op); // TODO make this closer to how it'd actually function?
            self.state = State::Run(instruction);
          }
        }
      },
    }

    let c = match &mut self.state {
      State::Fetch => return Err(Error::Impossible(op, "It should be literally impossible for ControlLogic.state to be Fetch.")),
      State::Init => return Err(Error::Impossible(op, "It should be literally impossible for ControlLogic.state to be Init.")),
      State::Run(instruction) => {
        match instruction.next() {
          Some(c) if flags.test(c) => c,
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
