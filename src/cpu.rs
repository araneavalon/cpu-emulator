
use crate::math::*;
use crate::bus;
// use crate::control;
use crate::error::Error;

use crate::bus::Device;

use crate::control_logic::ControlLogic;
use crate::program_counter::ProgramCounter;
use crate::stack_pointer::StackPointer;
use crate::register::Register;
use crate::h_register::HRegister;
use crate::l_register::LRegister;
use crate::memory::Memory;
use crate::flags_register::FlagsRegister;
use crate::alu::Alu;


#[derive(Debug)]
pub struct Cpu {
  pub ir: ControlLogic,
  pub pc: ProgramCounter,
  pub sp: StackPointer,

  pub a: Register,
  pub b: Register,
  pub x: Register,
  pub y: Register,

  pub h: HRegister,
  pub l: LRegister,

  pub memory: Memory,

  pub flags: FlagsRegister,
  pub alu: Alu,
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

      h: HRegister::new(),
      l: LRegister::new(),

      memory: Memory::new(),

      flags: FlagsRegister::new(),
      alu: Alu::new(),
    }
  }

  fn update(&mut self) -> Result<(), Error> {
    let control = self.ir.get_control();

    self.ir.update(control.Instruction)?;
    self.pc.update(control.ProgramCounter)?;
    self.sp.update(control.StackPointer)?;

    self.a.update(control.A)?;
    self.b.update(control.B)?;
    self.x.update(control.X)?;
    self.y.update(control.Y)?;

    self.h.update(control.H)?;
    self.l.update(control.L)?;

    self.memory.update(control.Memory)?;

    self.flags.update(control.FlagsRegister)?;
    self.alu.update(control.Alu)?;

    Ok(())
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
        (Some(bus::Addr::Full(_)), Some(bus::Addr::Full(_))) => return Err(Error::BusConflict(vec![String::from("addr:HL")])),
        (Some(bus::Addr::Full(_)), Some(bus::Addr::High(_))) => return Err(Error::BusConflict(vec![String::from("addr:HL,H")])),
        (Some(bus::Addr::Full(_)), Some(bus::Addr::Low(_)))  => return Err(Error::BusConflict(vec![String::from("addr:HL,L")])),
        (Some(bus::Addr::High(_)), Some(bus::Addr::Full(_))) => return Err(Error::BusConflict(vec![String::from("addr:H,HL")])),
        (Some(bus::Addr::Low(_)), Some(bus::Addr::Full(_)))  => return Err(Error::BusConflict(vec![String::from("addr:L,HL")])),
        (Some(bus::Addr::High(_)), Some(bus::Addr::High(_))) => return Err(Error::BusConflict(vec![String::from("addr:H")])),
        (Some(bus::Addr::Low(_)), Some(bus::Addr::Low(_)))   => return Err(Error::BusConflict(vec![String::from("addr:L")])),
        (Some(bus::Addr::High(h)), Some(bus::Addr::Low(l)))  => state.addr = Some(bus::Addr::Full(from_bytes(&[h, l]))),
        (Some(bus::Addr::Low(l)), Some(bus::Addr::High(h)))  => state.addr = Some(bus::Addr::Full(from_bytes(&[h, l]))),
        (Some(_), None)                                      => state.addr = s.addr,
        (None, _)                                            => (),
      }
    }
    Ok(state)
  }

  fn bus_state(&mut self) -> Result<bus::State, Error> {
    let state = self.merge_states(vec![
      self.pc.read(),
      self.sp.read(),
      self.h.read(),
      self.l.read(),
      self.alu.read(),
    ])?;
    self.memory.set_addr(&state)?;
    self.merge_states(vec![
      state,
      self.flags.read(),
      self.a.read(),
      self.b.read(),
      self.x.read(),
      self.y.read(),
      self.memory.read(),
    ])
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    self.ir.clk(state)?;
    self.pc.clk(state)?;
    self.sp.clk(state)?;

    self.a.clk(state)?;
    self.b.clk(state)?;
    self.x.clk(state)?;
    self.y.clk(state)?;

    self.h.clk(state)?;
    self.l.clk(state)?;

    self.memory.clk(state)?;

    self.flags.clk(state)?;
    self.alu.clk(state)?;

    Ok(())
  }

  fn run(&mut self, hz: u64) -> Result<(), Error> {
    // Two phase clock, therefore duration is halved.
    let ns = std::time::Duration::from_nanos((1_000_000_000 / 2) / hz);
    loop {
      self.update()?;
      std::thread::sleep(ns);
      let state = self.bus_state()?;
      self.clk(&state)?;
      std::thread::sleep(ns);
    }
  }
}
