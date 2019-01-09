
use crate::bus;
use crate::control;
use crate::error::Error;

use crate::register::Register;
use crate::program_counter::ProgramCounter;
use crate::memory::Memory;


#[derive(Debug)]
pub struct Cpu {
  pub pc: ProgramCounter,

  pub a: Register,
  pub b: Register,
  pub x: Register,
  pub y: Register,

  pub memory: Memory,
}

impl Cpu {
  pub fn new() -> Cpu {
    Cpu {
      pc: ProgramCounter::new(),

      a: Register::new(),
      b: Register::new(),
      x: Register::new(),
      y: Register::new(),

      memory: Memory::new(),
    }
  }

  fn merge_states(&self, states: Vec<bus::State>) -> Result<bus::State, Error> {
    let state = bus::State { data: None, addr: None };
    for let s in states.iter() {
      if let Some(_) = s.data {
        if let None = state.data {
          state.data = s.data;
        } else {
          // TODO ERROR
        }
      }
      if let Some(_) = s.addr {
        if let None = state.addr {
          state.addr = s.addr;
        } else {
          // TODO ERROR
        }
      }
    }
    Ok(state)
  }

  pub fn bus_state(&self) -> Result<bus::State, Error> {
    let state = self.merge_states(vec![
      self.pc.read(),
    ])?;
    self.memory.set_addr(&state);
    self.merge_states(vec![
      state,
      self.a.read(),
      self.b.read(),
      self.x.read(),
      self.y.read(),
      self.memory.read(),
    ])
  }
}
