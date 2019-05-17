
use std::fmt;
use std::time::Duration;
use std::thread;
use crate::error::{
  Result,
  Error,
};
use super::components::{
  BusComponent,
  AddressRegister,
  Alu,
  Flags,
  InstructionRegister,
  LinkRegister,
  ProgramCounter,
  RegisterFile,
  StackPointers,
};
use super::memory::Memory;
use super::io::{
  Screen,
  Keyboard,
};
use super::control::{
  ControlLogic,
  Control,
};


#[derive(Debug)]
pub struct Cpu {
  hz: f64,
  clock: Duration,
  halt: bool,
  c: Control,

  control: ControlLogic,

  a: AddressRegister,
  alu: Alu,
  flags: Flags,
  i: InstructionRegister,
  lr: LinkRegister,
  memory: Memory,
  pc: ProgramCounter,
  r: RegisterFile,
  s: StackPointers,
}

impl Cpu {
  pub fn new(hz: f64, rom: Vec<u16>) -> Result<Cpu> {
    Ok(Cpu {
      hz: hz,
      clock: Duration::from_nanos((1_000_000_000.0 / (hz * 2.0)) as u64),
      halt: false,
      c: Control::new(),

      control: ControlLogic::new()?,

      a: AddressRegister::new(),
      alu: Alu::new(),
      flags: Flags::new(),
      i: InstructionRegister::new(),
      lr: LinkRegister::new(),
      memory: Memory::new(rom),
      pc: ProgramCounter::new(),
      r: RegisterFile::new(),
      s: StackPointers::new(),
    })
  }

  fn components(&self) -> Vec<&dyn BusComponent> {
    vec![
      &self.a, &self.alu, &self.flags, &self.i,
      &self.lr, &self.memory, &self.pc, &self.r, &self.s,
    ]
  }

  fn components_mut(&mut self) -> Vec<&mut dyn BusComponent> {
    vec![
      &mut self.a, &mut self.alu, &mut self.flags, &mut self.i,
      &mut self.lr, &mut self.memory, &mut self.pc, &mut self.r, &mut self.s,
    ]
  }

  fn set_control(&mut self, c: Control) {
    for i in self.components_mut() {
      i.set_control(c);
    }
    self.c = c;
  }

  fn load(&mut self, value: Option<u16>) -> Result<()> {
    if let Some(value) = value {
      for i in self.components_mut() {
        i.load(value)?;
      }
    }
    Ok(())
  }

  fn data(&mut self) -> Result<Option<u16>> {
    let mut out = None;
    for component in self.components() {
      match (out, component.data()?) {
        (Some(_), Some(_)) => return Err(Error::DataBusConflict(self.i.get(), component.name())),
        (None, data) => out = data,
        (_, None) => (),
      }
    }
    match (out, self.i.get()) {
      (Some(value), _) => Ok(Some(value)),
      (None, 0x0000) => Ok(None), // NOP
      (None, 0x0080) => Ok(None), // HLT
      (None, _) if self.c.alu.set_flags => Ok(None), // CMP,CPN,TEST
      (None, op) => Err(Error::DataBusUnused(op)),
    }
  }

  fn address(&self) -> Result<u16> {
    let mut out = None;
    for component in self.components() {
      match (out, component.address()?) {
        (Some(_), Some(_)) => return Err(Error::AddressBusConflict(self.i.get(), component.name())),
        (None, address) => out = address,
        (_, None) => (),
      }
    }
    match out {
      None => Err(Error::AddressBusUnused(self.i.get())),
      Some(value) => Ok(value),
    }
  }

  fn half_cycle(&mut self) -> Result<()> {
    let c = self.control.decode(self.i.get(), &self.flags, &mut self.i)?;
    self.set_control(c);
    self.memory.set_address(self.address()?);
    self.lr.link(self.pc.link());
    Ok(())
  }

  fn cycle(&mut self) -> Result<()> {
    let data = self.data()?;
    self.load(data)?;
    self.flags.set_alu(self.alu.get_flags());
    Ok(())
  }

  pub fn hz(&self) -> f64 {
    self.hz
  }

  pub fn run(&mut self, cycles: u32) -> Result<()> {
    if !self.halt {
      for cycle in 0..cycles {
        thread::sleep(self.clock);
        self.half_cycle()?;

        thread::sleep(self.clock);
        self.cycle()?;

        if self.c.halt {
          self.halt = true;
          thread::sleep(self.clock * (cycles - cycle - 1) * 2);
          break;
        }
      }
      Ok(())
    } else {
      thread::sleep(self.clock * cycles * 2);
      Ok(())
    }
  }

  pub fn pause(&mut self) {
    self.halt = !self.halt;
  }

  pub fn screen(&self) -> Result<&Screen> {
    self.memory.screen()
  }

  pub fn keyboard(&mut self) -> Result<&mut Keyboard> {
    self.memory.keyboard()
  }

  pub fn interrupt(&mut self, interrupt: u16) -> Result<()> {
    self.control.interrupt(interrupt)
  }
}

impl fmt::Display for Cpu {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{c}\n{r}\n\n{pc}\n{lr}\n{s}\n{a}\n{mem}\n\n{i}\n{f}\n\n{alu}\n\n",
      c=self.control, r=self.r, pc=self.pc, lr=self.lr, s=self.s, a=self.a, mem=self.memory, i=self.i, f=self.flags, alu=self.alu)
  }
}
