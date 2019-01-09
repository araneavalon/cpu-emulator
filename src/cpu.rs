
use crate::bus;
// use crate::control;
use crate::error::Error;

use crate::bus::Device;
use crate::register::Register;
use crate::program_counter::ProgramCounter;
use crate::stack_pointer::StackPointer;
use crate::memory::Memory;
use crate::control_logic::ControlLogic;


#[derive(Debug)]
pub struct Cpu {
  pub ir: ControlLogic,
  pub pc: ProgramCounter,
  pub sp: StackPointer,

  pub a: Register,
  pub b: Register,
  pub x: Register,
  pub y: Register,

  pub memory: Memory,
}

impl Cpu {
  pub fn new() -> Cpu {
    Cpu {
      ir: ControlLogic::new(),
      pc: ProgramCounter::new(),
      sp: StackPointer::new(),

      a: Register::new(),
      b: Register::new(),
      x: Register::new(),
      y: Register::new(),

      memory: Memory::new(),
    }
  }

  fn update(&self) -> Result<(), Error> {
    let control = self.ir.get_control();

    self.ir.update(control.Instruction)?;
    self.pc.update(control.ProgramCounter)?;
    self.sp.update(control.StackPointer)?;

    self.a.update(control.A)?;
    self.b.update(control.B)?;
    self.x.update(control.X)?;
    self.y.update(control.Y)?;

    self.memory.update(control.Memory)?;

    Ok(())
  }

  fn merge_states(&self, states: Vec<bus::State>) -> Result<bus::State, Error> {
    let state = bus::State { data: None, addr: None };
    for s in states.iter() {
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

  fn bus_state(&self) -> Result<bus::State, Error> {
    let state = self.merge_states(vec![
      self.pc.read(),
      self.sp.read(),
    ])?;
    self.memory.set_addr(&state)?;
    self.merge_states(vec![
      state,
      self.a.read(),
      self.b.read(),
      self.x.read(),
      self.y.read(),
      self.memory.read(),
    ])
  }

  fn clk(&self, state: &bus::State) -> Result<(), Error> {
    self.ir.clk(state)?;
    self.pc.clk(state)?;
    self.sp.clk(state)?;

    self.a.clk(state)?;
    self.b.clk(state)?;
    self.x.clk(state)?;
    self.y.clk(state)?;

    self.memory.clk(state)?;

    Ok(())
  }


  fn run(&self, hz: u64) -> Result<(), Error> {
    // Two phase clock, therefore duration is halved.
    let ms = std::time::Duration::from_millis((1000 / 2) / hz);
    loop {
      self.update()?;
      std::thread::sleep(ms);
      let state = self.bus_state()?;
      self.clk(&state)?;
      std::thread::sleep(ms);
    }
  }
}
