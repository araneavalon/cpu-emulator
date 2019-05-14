
use std::fmt;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Value<'a> {
  Const(u16),
  Star,
  Label(&'a str),
  Relative(bool, &'a str),
}
impl<'a> fmt::Display for Value<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Value::Const(value)       => write!(f, "Const({})", value),
      Value::Star               => write!(f, "Label(*)"),
      Value::Label(label)       => write!(f, "Label({})", label),
      Value::Relative(true, c)  => write!(f, "Relative({}+)", c),
      Value::Relative(false, c) => write!(f, "Relative({}-)", c),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Expression<'a> {
  Value(Value<'a>),
  Add(Value<'a>, Value<'a>),
  Sub(Value<'a>, Value<'a>),
}
impl<'a> fmt::Display for Expression<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Expression::Value(value) => write!(f, "{}", value),
      Expression::Add(a, b) => write!(f, "Add({}, {})", a, b),
      Expression::Sub(a, b) => write!(f, "Sub({}, {})", a, b),
    }
  }
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
}
impl From<Register> for u16 {
  fn from(value: Register) -> u16 {
    value as u16
  }
}
impl From<&Register> for u16 {
  fn from(value: &Register) -> u16 {
    u16::from(*value)
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StackRegister {
  Zero = 0,
  One,
}
impl From<StackRegister> for u16 {
  fn from(value: StackRegister) -> u16 {
    value as u16
  }
}
impl From<&StackRegister> for u16 {
  fn from(value: &StackRegister) -> u16 {
    u16::from(*value)
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ProgramRegister {
  ProgramCounter = 0,
  LinkRegister,
}
impl From<ProgramRegister> for u16 {
  fn from(value: ProgramRegister) -> u16 {
    value as u16
  }
}
impl From<&ProgramRegister> for u16 {
  fn from(value: &ProgramRegister) -> u16 {
    u16::from(*value)
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AnyRegister {
  Register(Register),
  Stack(StackRegister),
  Program(ProgramRegister),
}
impl From<AnyRegister> for u16 {
  fn from(value: AnyRegister) -> u16 {
    match value {
      AnyRegister::Register(r) => u16::from(r),
      AnyRegister::Stack(s) => u16::from(s),
      AnyRegister::Program(p) => 0x0002 | u16::from(p),
    }
  }
}
impl From<&AnyRegister> for u16 {
  fn from(value: &AnyRegister) -> u16 {
    u16::from(*value)
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Argument<'a> {
  Constant(Expression<'a>),
  Variable(Expression<'a>),
  Direct(Register),
  Indirect(Register),
  Indexed(Register, Register),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Condition {
  Always(bool),
  Zero(bool),
  Sign(bool),
  Carry(bool),
  Overflow(bool),
  CarryAndNotZero(bool),
  OverflowAndNotZero(bool),
}
impl From<Option<Condition>> for Condition {
  fn from(value: Option<Condition>) -> Condition {
    match value {
      Some(c) => c,
      None => Condition::Always(false),
    }
  }
}
impl From<Condition> for u16 {
  fn from(value: Condition) -> u16 {
    match value {
      Condition::Always(n)             => ((n as u16) << 11),
      Condition::Zero(n)               => ((n as u16) << 11) | 2,
      Condition::Sign(n)               => ((n as u16) << 11) | 3,
      Condition::Carry(n)              => ((n as u16) << 11) | 4,
      Condition::Overflow(n)           => ((n as u16) << 11) | 5,
      Condition::CarryAndNotZero(n)    => ((n as u16) << 11) | 6,
      Condition::OverflowAndNotZero(n) => ((n as u16) << 11) | 7,
    }
  }
}
impl From<&Condition> for u16 {
  fn from(value: &Condition) -> u16 {
    u16::from(*value)
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AluOp {
  Add = 0,
  And,
  Cmp,
  Sub,
  Cpn,
  Sbn,
  Or,
  Xor
}
impl AluOp {
  pub fn is_short(&self) -> bool {
    match self {
      AluOp::Add |
      AluOp::Cmp |
      AluOp::Cpn => true,
      _ => false,
    }
  }
}
impl From<AluOp> for u16 {
  fn from(value: AluOp) -> u16 {
    value as u16
  }
}
impl From<&AluOp> for u16 {
  fn from(value: &AluOp) -> u16 {
    *value as u16
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOp {
  Not,
  Neg,
  Sl,
  Asr,
  Lsr,
}
impl From<UnaryOp> for u16 {
  fn from(value: UnaryOp) -> u16 {
    match value {
      UnaryOp::Not => 0,
      UnaryOp::Neg => 1,
      UnaryOp::Sl  => 4,
      UnaryOp::Lsr => 6,
      UnaryOp::Asr => 7,
    }
  }
}
impl From<&UnaryOp> for u16 {
  fn from(value: &UnaryOp) -> u16 {
    u16::from(*value)
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum JumpArgument<'a> {
  Argument(Argument<'a>),
  Stack(StackRegister),
  LinkRegister,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Op<'a> {
  Alu(AluOp, Register, Argument<'a>),
  Unary(UnaryOp, Register),

  Test(Register, Value<'a>),
  Set(Register, Value<'a>, bool),
  SetFlags(Value<'a>, bool),

  Load(bool, AnyRegister, Argument<'a>),

  Stack(bool, StackRegister, [bool; 10]),

  Jump(Condition, bool, JumpArgument<'a>),

  Interrupt(bool, Value<'a>),
  Nop(bool),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Symbol<'a> {
  Import(usize, &'a str),
  Define(usize, &'a str, Expression<'a>),
  Star(usize, u16),
  Word(usize, Expression<'a>),
  Label(usize, &'a str),
  Relative(usize, &'a str),
  Op(usize, Op<'a>),
  Comment(usize, &'a str),
}
