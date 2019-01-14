
use std::fmt;

use crate::bus;
use crate::control;
use crate::instructions;
use crate::error::Error;


fn init_micro() -> Vec<control::Control> {
  let mut c = control::Control::new();
  c.Instruction.Vector = Some(crate::memory::START_ADDRESS);
  c.ProgramCounter.Addr = control::ReadWrite::Read;
  vec![c]
}

#[derive(PartialEq, Eq)]
pub struct ControlLogic<T: instructions::Set> {
  instructions: Box<T>,
  control: control::Instruction,
  register: u8,

  fetch: bool,
  cycle: u8,
  pc: Option<control::IncDec>,
  sp: Option<control::IncDec>,
  micro: Vec<control::Control>,
}

impl<T: instructions::Set> ControlLogic<T> {
  pub fn new(instructions: Box<T>) -> ControlLogic<T> {
    ControlLogic {
      instructions: instructions,
      control: control::Instruction::new(),
      register: 0x00,

      fetch: false,
      cycle: 0,
      pc: None,
      sp: None,
      micro: init_micro(),
    }
  }

  fn set_micro(&mut self, micro: instructions::Micro, flags: &control::Flags) {
    match micro {
      instructions::Micro::Code(c) => {
        self.micro = c;
      },
      instructions::Micro::Compress(mut c) => {
        let l = c.remove(c.len() - 1);
        self.micro = c;
        self.pc = Some(l.ProgramCounter.Count);
        self.sp = Some(l.StackPointer.Count);
      },
      instructions::Micro::Branch(flag, t, f) => {
        if flags[&flag] {
          self.set_micro(*t, flags);
        } else {
          self.set_micro(*f, flags);
        }
      },
    }
  }

  pub fn get_control(&mut self, flags: &control::Flags) -> control::Control {
    self.cycle += 1;

    if self.micro.len() <= 0 {
      if self.fetch {
        self.set_micro(self.instructions.get(self.register), flags);
        self.fetch = false;
      }
    }

    if self.micro.len() <= 0 {
      if !self.fetch {
        self.cycle = 0;

        self.set_micro(self.instructions.fetch(), flags);
        if let Some(pc) = self.pc {
          self.micro[0].ProgramCounter.Count = pc;
        }
        if let Some(sp) = self.sp {
          self.micro[0].StackPointer.Count = sp;
        }

        self.fetch = true;
      }
    }

    self.micro.remove(0)
  }

  pub fn halt(&self) -> bool {
    self.control.Halt
  }
}

impl<T: instructions::Set> bus::Device<control::Instruction> for ControlLogic<T> {
  fn update(&mut self, control: control::Instruction) -> Result<(), Error> {
    self.control = control;
    Ok(())
  }

  fn read(&self) -> Result<bus::State, Error> {
    Ok(bus::State {
      data: None,
      addr: self.control.Vector,
    })
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    if let control::Read::Read = self.control.Data {
      self.register = state.read_data()?;
    }
    Ok(())
  }
}

impl<T: instructions::Set> fmt::Display for ControlLogic<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "ControlLogic({:#04X}) cycle={}", self.register, self.cycle)
  }
}

impl<T: instructions::Set> fmt::Debug for ControlLogic<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "ControlLogic({:#04X}) cycle={}", self.register, self.cycle)
  }
}
