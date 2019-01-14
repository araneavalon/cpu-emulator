
use std::fmt;

use crate::bus;
use crate::error::Error;

use crate::bus::Device;

use crate::instructions;
use crate::control_logic::ControlLogic;

use crate::program_counter::ProgramCounter;
use crate::stack_pointer::StackPointer;
use crate::register::Register;
use crate::address_register::AddressRegister;
use crate::memory::Memory;
use crate::flags_register::FlagsRegister;
use crate::alu::Alu;


#[derive(Debug)]
pub struct Cpu<T: instructions::Set> {
  halt: bool,

  ir: ControlLogic<T>,
  pc: ProgramCounter,
  sp: StackPointer,

  a: Register,
  b: Register,
  x: Register,
  y: Register,

  address: AddressRegister,

  memory: Memory,

  flags: FlagsRegister,
  alu: Alu,
}

impl<T: instructions::Set> Cpu<T> {
  pub fn new(instructions: Box<T>) -> Cpu<T> {
    Cpu {
      halt: false,

      ir: ControlLogic::new(instructions),
      pc: ProgramCounter::new(),
      sp: StackPointer::new(),

      a: Register::new(),
      b: Register::new(),
      x: Register::new(),
      y: Register::new(),

      address: AddressRegister::new(),

      memory: Memory::new(),

      flags: FlagsRegister::new(),
      alu: Alu::new(),
    }
  }

  pub fn resume(&mut self) {
    self.halt = false;
  }

  fn update(&mut self) -> Result<bool, Error> {
    let control = self.ir.get_control(self.flags.get_flags());

    self.ir.update(control.Instruction)?;
    self.pc.update(control.ProgramCounter)?;
    self.sp.update(control.StackPointer)?;

    self.a.update(control.A)?;
    self.b.update(control.B)?;
    self.x.update(control.X)?;
    self.y.update(control.Y)?;

    self.address.update(control.AddressRegister)?;

    self.memory.update(control.Memory)?;

    self.flags.update(control.FlagsRegister)?;
    self.alu.update(control.Alu)?;

    Ok(self.ir.halt())
  }

  fn merge_states(&self, states: Vec<bus::State>) -> Result<bus::State, Error> {
    let mut state = bus::State { data: None, addr: None };
    for s in states.iter() {
      match (s.data, state.data) {
        (Some(_), Some(_)) => return Err(Error::BusConflict(vec![String::from("data")])),
        (Some(_), None)    => state.data = s.data,
        (None, _)          => (),
      }
      match (s.addr, state.addr) {
        (Some(_), Some(_)) => return Err(Error::BusConflict(vec![String::from("addr")])),
        (Some(_), None)    => state.addr = s.addr,
        (None, _)          => (),
      }
    }
    Ok(state)
  }

  fn bus_state(&mut self) -> Result<bus::State, Error> {
    let state = self.merge_states(vec![
      self.ir.read()?,
      self.pc.read()?,
      self.sp.read()?,
      self.address.read()?,
      self.alu.read()?,
    ])?;

    self.memory.set_addr(&state)?;

    let state = self.merge_states(vec![
      state,
      self.flags.read()?,
      self.a.read()?,
      self.b.read()?,
      self.x.read()?,
      self.y.read()?,
      self.memory.read()?,
    ])?;

    Ok(state)
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    self.ir.clk(state)?;
    self.pc.clk(state)?;
    self.sp.clk(state)?;

    self.a.clk(state)?;
    self.b.clk(state)?;
    self.x.clk(state)?;
    self.y.clk(state)?;

    self.address.clk(state)?;

    self.memory.clk(state)?;

    self.flags.clk(state)?;
    self.alu.clk(state)?;

    self.flags.set_flags(self.alu.get_flags());

    Ok(())
  }

  pub fn run(&mut self, hz: u64) -> Result<(), Error> {
    if self.halt {
      return Ok(());
    }
    // Two phase clock, therefore duration is halved.
    let ns = std::time::Duration::from_nanos((1_000_000_000 / 2) / hz);
    loop {
      let halt = self.update()?;
      std::thread::sleep(ns);

      let state = self.bus_state()?;
      self.clk(&state)?;
      std::thread::sleep(ns);

      if !halt {
        println!("{}\n\n", self);
      }

      if halt {
        self.halt = true;
        return Ok(());
      }
    }
  }
}

impl<T: instructions::Set> fmt::Display for Cpu<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, " IR={}\n PC={}\n SP={}\n  A={}\n  B={}\n  X={}\n  Y={}\n HL={}\nMEM={}\n  F={}\nALU={}",
      self.ir, self.pc, self.sp, self.a, self.b, self.x, self.y,
      self.address, self.memory, self.flags, self.alu)
  }
}
