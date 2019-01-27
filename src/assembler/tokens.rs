
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operand {
  Star,
  Number(u16),
  Name(String),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
  Add(Operand, Operand),
  Sub(Operand, Operand),
  High(Operand),
  Low(Operand),
  Unary(Operand),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Flag { Z, C, V, S, I }

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Register { A, B, X, Y }

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Address {
  Direct(Expression),
  Indirect(Expression),
  Indexed(Expression, Register),
  IndirectIndexed(Expression, Register),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Argument {
  Byte(Expression),
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
pub enum Directive {
  Section(u16),
  Byte(Vec<Expression>),
  Word(Vec<Expression>),
  Define(String, Expression)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
  Directive(Directive),
  Label(String),
  Op(Op),
}
