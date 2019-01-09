
#[derive(Debug, PartialEq, Eq)]
pub enum Read {
  Read,
  None,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Write {
  Write,
  None,
}
#[derive(Debug, PartialEq, Eq)]
pub enum ReadWrite {
  Read,
  Write,
  None,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProgramCounterCount {
  Increment,
  Carry,
  Borrow,
  None,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StackPointerCount {
  Increment,
  Decrement,
  None,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AluSelect {
  Zero,
  One,
  Value,
  Invert,
}
#[derive(Debug, PartialEq, Eq)]
pub enum AluInput {
  Zero,
  Data,
  Addr,
}
#[derive(Debug, PartialEq, Eq)]
pub enum AluRotateDirection {
  Left,
  Right,
}
#[derive(Debug, PartialEq, Eq)]
pub enum AluOperation {
  Add {
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


pub trait Trait {}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
pub struct Instruction {
  pub Data: Read,
}
impl Instruction {
  pub fn new() -> Instruction {
    Instruction {
      Data: Read::None,
    }
  }
}
impl Trait for Instruction {}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq)]
pub struct ProgramCounter {
  pub DataH: ReadWrite,
  pub DataL: ReadWrite,
  pub Addr: ReadWrite,
  pub Count: ProgramCounterCount,
}
impl ProgramCounter {
  pub fn new() -> ProgramCounter {
    ProgramCounter {
      DataH: ReadWrite::None,
      DataL: ReadWrite::None,
      Addr: ReadWrite::None,
      Count: ProgramCounterCount::None,
    }
  }
}
impl Trait for ProgramCounter {}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq)]
pub struct StackPointer {
  pub Addr: Write,
  pub Count: StackPointerCount,
}
impl StackPointer {
  pub fn new() -> StackPointer {
    StackPointer {
      Addr: Write::None,
      Count: StackPointerCount::None,
    }
  }
}
impl Trait for StackPointer {}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
pub struct Alu {
  pub Temp: Read,
  pub TempSelect: AluSelect,
  pub Input: AluInput,
  pub Operation: AluOperation,
  pub Output: Write,
  pub Data: Write,
  pub Addr: Write,
}
impl Alu {
  pub fn new() -> Alu {
    Alu {
      Temp: Read::None,
      TempSelect: AluSelect::Zero,
      Input: AluInput::Zero,
      Operation: AluOpteration::Add {
        Carry: AluSelect::Zero,
      },
      Output: Write::None,
      Data: Write::None,
      Addr: Write::None,
    }
  }
}
impl Trait for Alu {}


#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq)]
pub struct Control {
  pub A: Register,
  pub B: Register,
  pub X: Register,
  pub Y: Register,

  pub Instruction: Instruction,
  pub ProgramCounter: ProgramCounter,
  pub StackPointer: StackPointer,

  pub Memory: Memory,

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

      Memory: Memory::new(),

      Alu: Alu::new(),
    }
  }
}
