
use std::collections::HashMap;
use std::fmt;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Read {
  Read,
  None,
}
impl fmt::Display for Read {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Read::Read => write!(f, "Read"),
      Read::None => write!(f, "None"),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Write {
  Write,
  None,
}
impl fmt::Display for Write {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Write::Write => write!(f, "Write"),
      Write::None => write!(f, "None"),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReadWrite {
  Read,
  Write,
  None,
}
impl fmt::Display for ReadWrite {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ReadWrite::Read => write!(f, "Read"),
      ReadWrite::Write => write!(f, "Write"),
      ReadWrite::None => write!(f, "None"),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IncDec {
  Increment,
  Decrement,
  None,
}
impl fmt::Display for IncDec {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      IncDec::Increment => write!(f, "Increment"),
      IncDec::Decrement => write!(f, "Decrement"),
      IncDec::None => write!(f, "None"),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AluSelect {
  Zero,
  One,
  Value,
  Invert,
}
impl fmt::Display for AluSelect {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      AluSelect::Zero => write!(f, "Zero"),
      AluSelect::One => write!(f, "One"),
      AluSelect::Value => write!(f, "Value"),
      AluSelect::Invert => write!(f, "Invert"),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AluInput {
  Zero,
  Data,
  Addr,
}
impl fmt::Display for AluInput {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      AluInput::Zero => write!(f, "Zero"),
      AluInput::Data => write!(f, "Data"),
      AluInput::Addr => write!(f, "Address"),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AluRotateDirection {
  Left,
  Right,
}
impl fmt::Display for AluRotateDirection {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      AluRotateDirection::Left => write!(f, "Left"),
      AluRotateDirection::Right => write!(f, "Right"),
    }
  }
}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AluOperation {
  Add {
    SignExtend: bool,
    Carry: AluSelect
  },
  And,
  Or,
  Xor,
  Rotate {
    Direction: AluRotateDirection,
    Carry: bool
  },
}
impl fmt::Display for AluOperation {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    #[allow(non_snake_case)]
    match self {
      AluOperation::Rotate { Direction, Carry: false } => write!(f, "Rotate{}", Direction),
      AluOperation::Rotate { Direction, Carry: true } => write!(f, "Rotate{}Carry", Direction),
      AluOperation::Add { SignExtend, Carry } => write!(f, "Add(Carry={}, Signed={})", Carry, SignExtend ),
      AluOperation::And => write!(f, "And"),
      AluOperation::Or => write!(f, "Or"),
      AluOperation::Xor => write!(f, "Xor"),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Flag {
  Z,
  C,
  V,
  S,
  I,
}
impl fmt::Display for Flag {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    #[allow(non_snake_case)]
    match self {
      Flag::Z => write!(f, "Flag::Zero"),
      Flag::C => write!(f, "Flag::Carry"),
      Flag::V => write!(f, "Flag::Overflow"),
      Flag::S => write!(f, "Flag::Sign"),
      Flag::I => write!(f, "Flag::Interrupt"),
    }
  }
}
pub type Flags = HashMap<Flag, bool>;


pub trait Trait {}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Register {
  pub Data: ReadWrite,
}
impl Register {
  pub fn new() -> Register {
    Register {
      Data: ReadWrite::None,
    }
  }
}
impl Trait for Register {}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AddressRegister {
  pub DataH: Read,
  pub DataL: Read,
  pub Addr: ReadWrite,
}
impl AddressRegister {
  pub fn new() -> AddressRegister {
    AddressRegister {
      DataH: Read::None,
      DataL: Read::None,
      Addr: ReadWrite::None,
    }
  }
}
impl Trait for AddressRegister {}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FlagsRegister {
  pub Data: ReadWrite,
  pub Update: Read,
  pub I: Option<bool>,
  pub C: Option<bool>,
}
impl FlagsRegister {
  pub fn new() -> FlagsRegister {
    FlagsRegister {
      Data: ReadWrite::None,
      Update: Read::None,
      I: None,
      C: None,
    }
  }
}
impl Trait for FlagsRegister {}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Instruction {
  pub Data: Read,
  pub Halt: bool,
  pub Vector: Option<u16>,
}
impl Instruction {
  pub fn new() -> Instruction {
    Instruction {
      Data: Read::None,
      Halt: false,
      Vector: None,
    }
  }
}
impl Trait for Instruction {}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProgramCounter {
  pub DataH: Write,
  pub DataL: Write,
  pub Addr: ReadWrite,
  pub Count: IncDec,
}
impl ProgramCounter {
  pub fn new() -> ProgramCounter {
    ProgramCounter {
      DataH: Write::None,
      DataL: Write::None,
      Addr: ReadWrite::None,
      Count: IncDec::None,
    }
  }
}
impl Trait for ProgramCounter {}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StackPointer {
  pub Addr: Write,
  pub Count: IncDec,
  pub Reset: bool,
}
impl StackPointer {
  pub fn new() -> StackPointer {
    StackPointer {
      Addr: Write::None,
      Count: IncDec::None,
      Reset: false,
    }
  }
}
impl Trait for StackPointer {}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Memory {
  pub Data: ReadWrite,
}
impl Memory {
  pub fn new() -> Memory {
    Memory {
      Data: ReadWrite::None,
    }
  }
}
impl Trait for Memory {}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Alu {
  pub Temp: Read,
  pub TempSelect: AluSelect,
  pub Input: AluInput,
  pub Flags: Flags,
  pub Operation: AluOperation,
  pub Output: Write,
  pub FlagOutput: Write,
  pub Data: Write,
  pub Addr: Write,
}
impl Alu {
  pub fn new() -> Alu {
    Alu {
      Temp: Read::None,
      TempSelect: AluSelect::Zero,
      Input: AluInput::Zero,
      Flags: hash_map!{
        Flag::Z => false,
        Flag::C => false,
        Flag::V => false,
        Flag::S => false,
      },
      Operation: AluOperation::Add {
        SignExtend: false,
        Carry: AluSelect::Zero,
      },
      Output: Write::None,
      FlagOutput: Write::None,
      Data: Write::None,
      Addr: Write::None,
    }
  }
}
impl Trait for Alu {}


#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Control {
  pub A: Register,
  pub B: Register,
  pub X: Register,
  pub Y: Register,

  pub AddressRegister: AddressRegister,

  pub Instruction: Instruction,
  pub ProgramCounter: ProgramCounter,
  pub StackPointer: StackPointer,

  pub Memory: Memory,

  pub FlagsRegister: FlagsRegister,
  pub Alu: Alu,
}
impl Control {
  pub fn new() -> Control {
    Control {
      Instruction: Instruction::new(),
      ProgramCounter: ProgramCounter::new(),
      StackPointer: StackPointer::new(),

      A: Register::new(),
      B: Register::new(),
      X: Register::new(),
      Y: Register::new(),

      AddressRegister: AddressRegister::new(),

      Memory: Memory::new(),

      FlagsRegister: FlagsRegister::new(),
      Alu: Alu::new(),
    }
  }
}
