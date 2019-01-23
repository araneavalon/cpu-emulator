
use std::fmt;

use crate::bus;
use crate::error::Error;

use crate::bus::Device;

use crate::instructions;

use crate::memory_controller::MemoryController;
use crate::components::decoder::Decoder;
use crate::components::program_counter::ProgramCounter;
use crate::components::stack_pointer::StackPointer;
use crate::components::register::Register;
use crate::components::address_register::AddressRegister;
use crate::components::flags_register::FlagsRegister;
use crate::components::alu::Alu;


pub struct Cpu<T: instructions::Set> {
  hz: u64,
  tick_f: std::time::Duration,
  tick_h: std::time::Duration,
  halt: bool,

  ir: Decoder<T>,
  pc: ProgramCounter,
  sp: StackPointer,

  a: Register,
  b: Register,
  x: Register,
  y: Register,

  address: AddressRegister,

  flags: FlagsRegister,
  alu: Alu,

  pub memory: MemoryController,
}

impl<T: instructions::Set> Cpu<T> {
  pub fn new(hz: u64, instructions: Box<T>) -> Cpu<T> {
    Cpu {
      hz: hz,
      tick_f: std::time::Duration::from_nanos(1_000_000_000 / hz),
      tick_h: std::time::Duration::from_nanos(500_000_000 / hz),
      halt: false,

      ir: Decoder::new(instructions),
      pc: ProgramCounter::new(),
      sp: StackPointer::new(),

      a: Register::new(),
      b: Register::new(),
      x: Register::new(),
      y: Register::new(),

      address: AddressRegister::new(),

      flags: FlagsRegister::new(),
      alu: Alu::new(),

      memory: MemoryController::new(),
    }
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


  pub fn set_hz(&mut self, hz: u64) {
    self.hz = hz;
    self.tick_f = std::time::Duration::from_nanos(1_000_000_000 / hz);
    self.tick_h = std::time::Duration::from_nanos(500_000_000 / hz);
  }
  pub fn hz(&self) -> u64 {
    self.hz
  }

  pub fn halted(&self) -> bool {
    self.halt
  }
  pub fn resume(&mut self) {
    self.halt = false;
  }

  pub fn tick(&mut self) -> Result<bool, Error> {
    if self.halt {
      std::thread::sleep(self.tick_f);
    } else {
      self.halt = self.update()?;
      std::thread::sleep(self.tick_h);

      let state = self.bus_state()?;
      self.clk(&state)?;
      std::thread::sleep(self.tick_h);
    }
    Ok(self.halt)
  }

  pub fn run(&mut self, ticks: u64) -> Result<(), Error> {
    for _ in 0..ticks {
      self.tick()?;
    }
    Ok(())
  }
}

impl<T: instructions::Set> fmt::Display for Cpu<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "  A={}   B={}   X={}   Y={}\n PC={}  SP={} HL={}\n  F={} ALU={}\n IR={}\nMEM={}",
      self.a, self.b, self.x, self.y, self.pc, self.sp,
      self.address, self.flags, self.alu, self.ir, self.memory)
  }
}

impl<T: instructions::Set> fmt::Debug for Cpu<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, " IR={:?}\n PC={:?}\n SP={:?}\n  A={:?}\n  B={:?}\n  X={:?}\n  Y={:?}\n HL={:?}\nMEM={:?}\n  F={:?}\nALU={:?}",
      self.ir, self.pc, self.sp, self.a, self.b, self.x, self.y,
      self.address, self.memory, self.flags, self.alu)
  }
}
