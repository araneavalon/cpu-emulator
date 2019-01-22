
use std::fmt;

use crate::bus;
use crate::control;
use crate::instructions;
use crate::error::Error;


fn init_micro<T: instructions::Set>(instructions: &Box<T>) -> Vec<control::Control> {
  let mut c = control::Control::new();
  c.Instruction.Vector = Some(instructions.start());
  c.ProgramCounter.Addr = control::ReadWrite::Read;
  vec![c]
}

#[derive(Debug, PartialEq, Eq)]
enum State {
  Init,
  Fetch,
  Run(u8),
}

impl fmt::Display for State {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      State::Init => write!(f, "Init"),
      State::Fetch => write!(f, "Fetch"),
      State::Run(cycle) => write!(f, "Run(cycle={})", cycle),
    }
  }
}


#[derive(PartialEq, Eq)]
pub struct ControlLogic<T: instructions::Set> {
  instructions: Box<T>,
  control: control::Instruction,
  register: u8,

  state: State,
  pc: Option<control::IncDec>,
  sp: Option<control::IncDec>,
  micro: Vec<control::Control>,
}

impl<T: instructions::Set> ControlLogic<T> {
  pub fn new(instructions: Box<T>) -> ControlLogic<T> {
    let micro = init_micro(&instructions);
    ControlLogic {
      instructions: instructions,
      control: control::Instruction::new(),
      register: 0x00,

      state: State::Init,
      pc: None,
      sp: None,
      micro: micro,
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
        if l.ProgramCounter.Count != control::IncDec::None {
          self.pc = Some(l.ProgramCounter.Count);
        }
        if l.StackPointer.Count != control::IncDec::None {
          self.sp = Some(l.StackPointer.Count);
        }
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
    if self.micro.len() <= 0 {
      // Check Fetch -> Run transition first, because a 0-cycle instruction
      // needs to immediately transition back to Fetch.
      if let State::Fetch = self.state {
        self.set_micro(self.instructions.get(self.register), flags);
        self.state = State::Run(0);
      }
    } else if let State::Run(cycle) = self.state {
      // If more microcode to execute, increase cycle count.
      self.state = State::Run(cycle + 1);
    }

    if self.micro.len() <= 0 {
      self.state = match self.state {
        State::Fetch => panic!("Empty fetch state can not occur at this point."),
        State::Init |
        State::Run(_) => {
          self.set_micro(self.instructions.fetch(), flags);
          if let Some(pc) = self.pc {
            self.micro[0].ProgramCounter.Count = pc;
            self.pc = None;
          }
          if let Some(sp) = self.sp {
            self.micro[0].StackPointer.Count = sp;
            self.sp = None;
          }
          State::Fetch
        },
      };
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
    let address = if let Some(address) = self.control.Vector {
      format!("0x{:04X}", address)
    } else {
      String::from("None")
    };
    write!(f, "0x  {:02X} [{}] Micro={} Address={}",
      self.register, self.state, self.micro.len(), address)
  }
}

impl<T: instructions::Set> fmt::Debug for ControlLogic<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let address = if let Some(address) = self.control.Vector {
      format!("0x{:04X}", address)
    } else {
      String::from("None")
    };
    write!(f, "0x  {:02X} [{:?}] (Micro={:?}, Address={}, Data={:?}, Halt={:?}) [ControlLogic]",
      self.register, self.state, self.micro.len(), address, self.control.Data, self.control.Halt)
  }
}
