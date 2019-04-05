
use crate::components::BusComponent;
use crate::components::{
  address_register::AddressRegister,
  alu::Alu,
  flags::Flags,
  instruction_register::InstructionRegister,
  link_register::LinkRegister,
  memory::Memory,
  program_counter::ProgramCounter,
  register_file::RegisterFile,
  stack_pointers::StackPointers,
};
use crate::control::{
  ControlLogic,
  Control,
};


#[derive(Debug)]
struct Cpu {
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
  pub fn new() -> Cpu {
    Cpu {
      control: ControlLogic::new(),

      a: AddressRegister::new(),
      alu: Alu::new(),
      flags: Flags::new(),
      i: InstructionRegister::new(),
      lr: LinkRegister::new(),
      memory: Memory::new(),
      pc: ProgramCounter::new(),
      r: RegisterFile::new(),
      s: StackPointers::new(),
    }
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
  }

  fn load(&mut self, value: u16) {
    for i in self.components_mut() {
      i.load(value);
    }
  }

  fn data(&self) -> u16 {
    let mut out = None;
    for i in self.components() {
      match out {
        None => out = i.data(),
        Some(_) => panic!("Attempted to write two things to the data bus at once."),
      }
    }
    match out {
      None => panic!("Nothing wrote to the data bus this cycle, check microcode?"),
      Some(value) => value,
    }
  }

  fn address(&self) -> u16 {
    let mut out = None;
    for i in self.components() {
      match out {
        None => out = i.address(),
        Some(_) => panic!("Attempted to write two things to the address bus at once."),
      }
    }
    match out {
      None => panic!("Nothing wrote to the address bus this cycle, shouldn't the default be A?"),
      Some(value) => value,
    }
  }

  pub fn pre_cycle(&mut self) {
    let c = self.control.decode(self.i.get(), &self.flags);
    self.set_control(c);
    self.memory.set_address(self.address());
    self.lr.link(self.pc.link());
  }

  pub fn cycle(&mut self) {
    self.load(self.data());
    self.flags.set(self.alu.get_flags());
  }
}
