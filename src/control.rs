
use std::collections::HashMap;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Read {
  Read,
  None,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Write {
  Write,
  None,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReadWrite {
  Read,
  Write,
  None,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IncDec {
  Increment,
  Decrement,
  None,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AluSelect {
  Zero,
  One,
  Value,
  Invert,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AluInput {
  Zero,
  Data,
  Addr,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AluRotateDirection {
  Left,
  Right,
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Flag {
  Z,
  C,
  V,
  S,
  I,
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
  pub Addr: Write,
}
impl AddressRegister {
  pub fn new() -> AddressRegister {
    AddressRegister {
      DataH: Read::None,
      DataL: Read::None,
      Addr: Write::None,
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
}
impl StackPointer {
  pub fn new() -> StackPointer {
    StackPointer {
      Addr: Write::None,
      Count: IncDec::None,
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
