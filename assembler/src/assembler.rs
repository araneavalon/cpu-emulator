
use std::collections::HashMap;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use super::symbols::*;
use super::error::{
  self,
  KindResult,
  ErrorKind,
};


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Address(u16, u16);

impl Address {
  fn correct(&mut self, correction: Address) {
    self.0 += correction.0;
    self.1 -= correction.1;
  }

  fn is_byte(&self) -> bool {
    (-128 <= (self.0 as i16)) && ((self.1 as i16) <= 127)
  }

  fn is_zero(&self) -> bool {
    self.1 <= 255
  }

  fn as_bitmask(&self) -> KindResult<u16> {
    if self.1 <= 15 {
      self.value()
    } else {
      Err(ErrorKind::OutOfRange(self.0, self.1, 0, 15))
    }
  }

  fn as_interrupt(&self) -> KindResult<u16> {
    if self.1 <= 7 {
      self.value()
    } else {
      Err(ErrorKind::OutOfRange(self.0, self.1, 0, 7))
    }
  }

  fn value(&self) -> KindResult<u16> {
    if self.0 != self.1 {
      Err(ErrorKind::InvalidAddress(self.0, self.1))
    } else {
      Ok(self.0)
    }
  }
}

impl Add for Address {
  type Output = Address;

  fn add(self, other: Address) -> Address {
    Address(self.0.wrapping_add(other.0), self.1.wrapping_add(other.1))
  }
}
impl AddAssign for Address {
  fn add_assign(&mut self, other: Address) {
    *self = self.add(other);
  }
}

impl Sub for Address {
  type Output = Address;

  fn sub(self, other: Address) -> Address {
    Address(self.0 - other.0, self.1 - other.1)
  }
}
impl SubAssign for Address {
  fn sub_assign(&mut self, other: Address) {
    *self = self.sub(other);
  }
}

impl From<u16> for Address {
  fn from(value: u16) -> Address {
    Address(value, value)
  }
}
impl From<&u16> for Address {
  fn from(value: &u16) -> Address {
    Address(*value, *value)
  }
}

impl From<usize> for Address {
  fn from(value: usize) -> Address {
    Address(value as u16, value as u16)
  }
}
impl From<&usize> for Address {
  fn from(value: &usize) -> Address {
    Address(*value as u16, *value as u16)
  }
}


type Operation = (u16, Option<u16>);

#[derive(Debug)]
struct Assembler<'a> {
  labels: HashMap<&'a str, usize>,
  relative: HashMap<&'a str, Vec<usize>>,
  symbols: Vec<(Address, Symbol<'a>)>,
  words: Vec<u16>,
}

impl<'a> Assembler<'a> {
  pub fn new(symbols: Vec<(Address, Symbol<'a>)>) -> Assembler {
    Assembler {
      labels: HashMap::new(),
      relative: HashMap::new(),
      symbols,
      words: Vec::new(),
    }
  }

  fn value(&self, index: usize, value: &Value<'a>) -> KindResult<Address> {
    match value {
      Value::Const(value) => Ok(Address::from(value)),
      Value::Star => Ok(self.symbols[index].0),
      Value::Label(label) => {
        match self.labels.get(label) {
          None => Err(ErrorKind::UnknownLabel(String::from(*label))),
          Some(index) => {
            match self.symbols[*index] {
              (_, Symbol::Define(_, _, expr)) => self.expression(*index, &expr),
              (address, _) => Ok(address),
            }
          },
        }
      },
      Value::Relative(direction, label) => {
        match self.relative.get(label) {
          None => Err(ErrorKind::UnknownRelative(String::from(*label), index)),
          Some(indexes) => {
            if *direction {
              for i in indexes.iter() {
                if *i > index {
                  return Ok(self.symbols[*i].0)
                }
              }
            } else {
              for i in indexes.iter().rev() {
                if *i < index {
                  return Ok(self.symbols[*i].0)
                }
              }
            }
            Err(ErrorKind::UnknownRelative(String::from(*label), index))
          },
        }
      },
    }
  }

  fn expression(&self, index: usize, expr: &Expression<'a>) -> KindResult<Address> {
    match expr {
      Expression::Value(value) => self.value(index, value),
      Expression::Add(a, b) => Ok(self.value(index, a)? + self.value(index, b)?),
      Expression::Sub(a, b) => Ok(self.value(index, a)? - self.value(index, b)?),
    }
  }

  fn op_correction(&self, index: usize, symbol: &Op<'a>) -> KindResult<Address> {
    let length = match symbol {
      Op::Alu(op, _, Argument::Constant(expr)) if op.is_short()        => Some(self.expression(index, expr)?.is_byte()),
      Op::Alu(op, _, Argument::Variable(expr)) if op.is_short()        => Some(self.expression(index, expr)?.is_zero()),
      Op::Load(_, _, Argument::Constant(expr))                         => Some(self.expression(index, expr)?.is_byte()),
      Op::Load(_, _, Argument::Variable(expr))                         => Some(self.expression(index, expr)?.is_zero()),
      Op::Jump(_, _, JumpArgument::Argument(Argument::Constant(expr))) => Some(self.expression(index, expr)?.is_byte()),
      Op::Jump(_, _, JumpArgument::Argument(Argument::Variable(expr))) => Some(self.expression(index, expr)?.is_zero()),
      _ => None,
    };
    match length {
      Some(true)  => Ok(Address(0, 1)),
      Some(false) => Ok(Address(1, 0)),
      None        => Ok(Address(0, 0)),
    }
  }

  fn argument(&self, index: usize, argument: &Argument<'a>) -> KindResult<Operation> {
    match argument {
      Argument::Indexed(base, index) => Ok((0x0200 | (u16::from(index) << 6) | (u16::from(base) << 3), None)),
      Argument::Variable(expr) => Ok((0x0140, Some(self.expression(index, expr)?.value()?))),
      Argument::Constant(expr) => Ok((0x0100, Some(self.expression(index, expr)?.value()?))),
      Argument::Indirect(src)  => Ok((0x0040 | (u16::from(src) << 3), None)),
      Argument::Direct(src)    => Ok((0x0000 | u16::from(src) << 3, None)),
    }
  }

  fn byte_argument(&self, index: usize, argument: &Argument<'a>) -> KindResult<Result<u16, Operation>> {
    match argument {
      Argument::Variable(expr) => {
        let value = self.expression(index, expr)?;
        if value.is_zero() {
          return Ok(Ok((value.value()? as u16) << 3))
        }
      },
      Argument::Constant(expr) => {
        let value = self.expression(index, expr)?;
        if value.is_byte() {
          return Ok(Ok((value.value()? as u16) << 3))
        }
      },
      _ => (),
    };
    Ok(Err(self.argument(index, argument)?))
  }

  fn assemble_alu(&self, index: usize, op: &AluOp, dest: &Register, argument: &Argument<'a>) -> KindResult<Operation> {
    let (opcode, arg, word) = if op.is_short() {
      match (self.byte_argument(index, argument)?, argument) {
        (Err((arg, word)), _) => (0x2000, arg, word),
        (Ok(arg), Argument::Variable(_)) => (0x8000, arg, None),
        (Ok(arg), Argument::Constant(_)) => (0xC000, arg, None),
        (Ok(_), _) => return Err(ErrorKind::Impossible("Only Constant or Variable arguments can be byte-width.")),
      }
    } else {
      let (arg, word) = self.argument(index, argument)?;
      (0x2000, arg, word)
    };
    Ok((opcode | (u16::from(op) << 10) | arg | u16::from(dest), word))
  }

  fn assemble_load(&self, index: usize, direction: bool, dest: &AnyRegister, argument: &Argument<'a>) -> KindResult<Operation> {
    if let AnyRegister::Register(dest) = dest {
      let (opcode, offset, arg, word) = {
        match (self.byte_argument(index, argument)?, argument) {
          (Err((arg, word)), _) => (0x9800, 10, arg, word),
          (Ok(arg), Argument::Variable(_)) => (0x7000, 11, arg, None),
          (Ok(arg), Argument::Constant(_)) if direction => (0xD000, 11, arg, None),
          (Ok(_), Argument::Constant(_)) => return Err(ErrorKind::StoreConstant),
          (Ok(_), _) => return Err(ErrorKind::Impossible("Only Constant or Variable arguments can be byte-width.")),
        }
      };
      Ok((opcode | ((direction as u16) << offset) | arg | u16::from(dest), word))
    } else {
      let (arg, word) = self.argument(index, argument)?;
      Ok((0x0800 | ((direction as u16) << 10) | (arg & 0x03C0) | ((arg & 0x0031) >> 3) | (u16::from(dest) << 3), word))
    }
  }

  fn assemble_stack(&self, direction: bool, stack: &StackRegister, registers: &[bool; 10]) -> KindResult<Operation> {
    let mut op = 0x1000 | ((direction as u16) << 10) | (u16::from(stack) << 9);
    for offset in 0..9 {
      op |= (registers[offset] as u16) << offset;
    }
    op |= (registers[9] as u16) << 11;
    Ok((op, None))
  }

  fn assemble_jump(&self, index: usize, condition: &Condition, link: bool, argument: &JumpArgument<'a>) -> KindResult<Operation> {
    let (opcode, offset, arg, word) = match argument {
      JumpArgument::LinkRegister => (0x4100, 10, 0x0000, None),
      JumpArgument::Stack(stack) => (0x4000, 10, u16::from(stack) << 9, None),
      JumpArgument::Argument(argument) => {
        match (self.byte_argument(index, argument)?, argument) {
          (Err((arg, word)), _) => (0x5000, 10, arg, word),
          (Ok(arg), Argument::Variable(_)) => (0xA000, 12, arg, None),
          (Ok(arg), Argument::Constant(_)) => (0xE000, 12, arg, None),
          (Ok(_), _) => return Err(ErrorKind::Impossible("Only Constant or Variable arguments can be byte-width.")),
        }
      },
    };
    Ok((opcode | ((link as u16) << offset) | arg | u16::from(condition), word))
  }

  fn assemble_interrupt(&self, index: usize, halt: bool, interrupt: &Value<'a>) -> KindResult<Operation> {
    Ok((0x0400 | (self.value(index, interrupt)?.as_interrupt()? << 3) | ((halt as u16) << 7), None))
  }

  fn assemble_op(&self, index: usize, op: &Op<'a>) -> KindResult<Operation> {
    match op {
      Op::Alu(op, register, argument) => self.assemble_alu(index, op, register, argument),
      Op::Unary(op, register) => Ok((0x6400 | (u16::from(op) << 3) | u16::from(register), None)),
      Op::Test(register, bit) => {
        Ok((0x6000 | (self.value(index, bit)?.as_bitmask()? << 3) | u16::from(register), None))
      },
      Op::Set(register, bit, value) => {
        Ok((0x6800 | (self.value(index, bit)?.as_bitmask()? << 3) | ((*value as u16) << 7) | u16::from(register), None))
      },
      Op::SetFlags(bit, value) => {
        Ok((0x6C00 | (self.value(index, bit)?.as_bitmask()? << 3) | ((*value as u16) << 7), None))
      },
      Op::Load(direction, register, argument) => self.assemble_load(index, *direction, register, argument),
      Op::Stack(direction, stack, registers) => self.assemble_stack(*direction, stack, registers),
      Op::Jump(condition, link, argument) => self.assemble_jump(index, condition, *link, argument),
      Op::Interrupt(halt, interrupt) => self.assemble_interrupt(index, *halt, interrupt),
      Op::Nop(halt) => Ok(((*halt as u16) << 7, None)),
    }
  }

  fn label(&mut self) -> error::Result<()> {
    for index in 0..self.symbols.len() {
      match self.symbols[index].1 {
        Symbol::Define(line, label, _) |
        Symbol::Label(line, label) => try_kind!(line, {
          match self.labels.get(label) {
            Some(_) => Err(ErrorKind::DuplicateLabel(String::from(label))),
            None => { self.labels.insert(label, index); Ok(()) },
          }
        }),
        Symbol::Relative(line, label) => try_kind!(line, {
          match self.relative.get_mut(&label) {
            Some(relative) => {
              if let Err(i) = relative.binary_search(&index) {
                relative.insert(i, index);
              }
            },
            None => { self.relative.insert(label, vec![index]); },
          }
          Ok(())
        }),
        _ => (),
      }
    }
    Ok(())
  }

  fn correct(&mut self) -> error::Result<()> {
    let mut correction = Address(0, 0);
    for index in 0..self.symbols.len() {
      self.symbols[index].0.correct(correction);

      match self.symbols[index].1 {
        Symbol::Op(line, op) => correction += try_kind!(line, self.op_correction(index, &op)),
        Symbol::Star(_, _) => correction = Address(0, 0),
        _ => (),
      }
    }
    Ok(())
  }

  fn assemble(&mut self) -> error::Result<()> {
    let mut words = Vec::new();

    for index in 0..self.symbols.len() {
      match self.symbols[index].1 {
        Symbol::Star(line, star) => {
          let address = try_kind!(line, self.symbols[index].0.value());
          if address != 0x0000 {
            for _ in address..star {
              words.push(0x0000);
            }
          }
        },
        Symbol::Word(line, expr) => {
          let value = try_kind!(line, self.expression(index, &expr));
          words.push(try_kind!(line, value.value()));
        },
        Symbol::Op(line, op) => {
          let (op, word) = try_kind!(line, self.assemble_op(index, &op));
          words.push(op);
          if let Some(word) = word {
            words.push(word);
          }
        },
        _ => (),
      }
    }

    self.words = words;
    Ok(())
  }

  fn words(self) -> Vec<u16> {
    self.words
  }
}


fn op_len(symbol: &Op) -> Address {
  match symbol {
    Op::Alu(op, _, Argument::Constant(_)) if op.is_short() => Address(1, 2),
    Op::Alu(op, _, Argument::Variable(_)) if op.is_short() => Address(1, 2),
    Op::Load(_, _, Argument::Constant(_)) => Address(1, 2),
    Op::Load(_, _, Argument::Variable(_)) => Address(1, 2),
    Op::Jump(_, _, JumpArgument::Argument(Argument::Constant(_))) => Address(1, 2),
    Op::Jump(_, _, JumpArgument::Argument(Argument::Variable(_))) => Address(1, 2),

    Op::Alu(_, _, Argument::Constant(_)) => Address(2, 2),
    Op::Alu(_, _, Argument::Variable(_)) => Address(2, 2),

    _ => Address(1, 1),
  }
}

fn address<'a>(symbols: Vec<Symbol<'a>>) -> Vec<(Address, Symbol<'a>)> {
  let mut out = Vec::new();

  let mut address = Address(0x0000, 0x0000);
  for symbol in symbols.into_iter() {
    let addr = address;
    match &symbol {
      Symbol::Word(_, _) => address += Address(1, 1),
      Symbol::Star(_, value) => address = Address::from(value),
      Symbol::Op(_, op) => address += op_len(op),
      _ => (),
    }
    out.push((addr, symbol));
  }

  out
}

pub fn assemble<'a>(symbols: Vec<Symbol<'a>>) -> error::Result<Vec<u16>> {
  let mut assembler = Assembler::new(address(symbols));
  assembler.label()?;
  assembler.correct()?;
  assembler.assemble()?;
  Ok(assembler.words())
}
