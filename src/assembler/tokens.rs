
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Flag { Z, C, V, S, I }

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Register { A, B, X, Y }

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AddressTarget {
  Label(String),
  Address(u16),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Address {
  Direct(AddressTarget),
  Indirect(AddressTarget),
  Indexed(AddressTarget, Register),
  IndirectIndexed(AddressTarget, Register),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Argument {
  Byte(u8),
  Register(Register),
  Address(Address),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Op {
  Nop,
  Hlt,
  Brk,
  Int,
  Set(Flag, bool),
  Call(Address),
  Ret,
  RetI,
  Jmp(Option<(Flag, bool)>, Address),
  Add(Register, Argument),
  AddC(Register, Argument),
  Sub(Register, Argument),
  SubC(Register, Argument),
  And(Register, Argument),
  Or(Register, Argument),
  Xor(Register, Argument),
  Cmp(Register, Argument),
  Inc(Register),
  Dec(Register),
  Neg(Register),
  Not(Register),
  Rr(Register),
  RrC(Register),
  Rl(Register),
  RlC(Register),
  Push(Register),
  Pop(Register),
  Ld(Argument, Argument),
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
  Label(String),
  Op(Op),
}
