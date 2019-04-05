
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Address {
  A,
  ProgramCounter,
  StackZero,
  StackOne,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Register {
  Zero = 0,
  One,
  Two,
  Three,
  Four,
  Five,
  Six,
  Seven,
  None,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RegisterFile {
  pub load: Register,
  pub out: Register,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ProgramRegister {
  pub load: bool,
  pub out: bool,
  pub increment: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct StackRegister {
  pub load: bool,
  pub out: bool,
  pub count: bool,
  pub direction: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BiRegister {
  pub load: bool,
  pub out: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LoadRegister {
  pub load: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IMode {
  None,
  SignedByte,
  UnsignedByte,
  WordOffset,
  Bitmask,
  Interrupt,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct InstructionRegister {
  pub load: bool,
  pub mode: IMode,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AluMode {
  Add,
  And,
  Or,
  Xor,
  Shift,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Alu {
  pub mode: AluMode,
  pub t: [LoadRegister; 2],
  pub out: bool,
  pub set_flags: bool,
  // ADD, AND, OR, XOR
  pub t0_zero: bool,
  pub t1_invert: bool,
  pub carry_invert: bool,
  // SHIFT
  pub direction: bool,
  pub word: bool,
  pub extend: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Memory {
  pub load: bool,
  pub out: bool,
  pub word: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Condition {
  Always,
  Zero,
  Sign,
  Carry,
  CarryNotZero,
  Overflow,
  OverflowNotZero,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Branch {
  pub negate: bool,
  pub condition: Condition,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Control {
  pub address: Address,
  pub register: RegisterFile,
  pub alu: Alu,
  pub flags: BiRegister,
  pub pc: ProgramRegister,
  pub lr: ProgramRegister,
  pub s: [StackRegister; 2],
  pub a: LoadRegister,
  pub i: InstructionRegister,
  pub memory: Memory,
  pub branch: Branch,
  pub link: bool,
  pub stack_sequence: bool,
  pub interrupt: bool,
  pub halt: bool,
}

impl Control {
  pub fn new() -> Control {
    Control {
      address: Address::A,
      register: RegisterFile { load: Register::None, out: Register::None, },
      alu: Alu {
        mode: AluMode::Add,
        t: [
          LoadRegister { load: false },
          LoadRegister { load: false },
        ],
        out: false,
        set_flags: false,
        t0_zero: false,
        t1_invert: false,
        carry_invert: false,
        direction: false,
        word: false,
        extend: false,
      },
      flags: BiRegister { load: false, out: false },
      pc: ProgramRegister { load: false, out: false, increment: false },
      lr: ProgramRegister { load: false, out: false, increment: false },
      s: [
        StackRegister { load: false, out: false, count: false, direction: false },
        StackRegister { load: false, out: false, count: false, direction: false },
      ],
      a: LoadRegister { load: false },
      i: InstructionRegister { load: false, mode: IMode::None },
      memory: Memory { load: false, out: false, word: true },
      branch: Branch { negate: false, condition: Condition::Always },
      link: false,
      stack_sequence: false,
      interrupt: false,
      halt: false,
    }
  }

  pub fn previous(&self, c: Control) -> Control {
    let mut s = self.clone();
    s.pc.increment = c.pc.increment;
    s.lr.increment = c.lr.increment;
    for i in 0..2 {
      if c.s[i].count & !c.s[i].direction {
        s.s[i].count = true;
        s.s[i].direction = false;
      } else if s.s[i].count & !s.s[i].direction {
        s.s[i].count = false;
      }
    }
    s
  }
}
