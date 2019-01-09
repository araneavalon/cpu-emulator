
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
pub struct ProgramCounter {
  pub DataH: ReadWrite,
  pub DataL: ReadWrite,
  pub Addr: Write,
  pub Count: ProgramCounterCount,
}
impl ProgramCounter {
  pub fn new() -> ProgramCounter {
    ProgramCounter {
      DataH: ReadWrite::None,
      DataL: ReadWrite::None,
      Addr: Write::None,
      Count: ProgramCounterCount::None,
    }
  }
}
impl Trait for ProgramCounter {}

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
pub struct Control {
  pub A: Register,
  pub B: Register,
  pub X: Register,
  pub Y: Register,

  pub ProgramCounter: ProgramCounter,

  pub Memory: Memory,
}
impl Control {
  pub fn new() -> Control {
    Control {
      A: Register::new(),
      B: Register::new(),
      X: Register::new(),
      Y: Register::new(),

      ProgramCounter: ProgramCounter::new(),

      Memory: Memory::new(),
    }
  }
}
