
use std::collections::HashMap;

use crate::math::*;
use super::error::{self, Error};
use super::tokens::*;


impl UnaryExpr {
  pub fn resolve(&self, address: u16, names: &HashMap<String, Value>) -> Result<Value, Error> {
    match self {
      UnaryExpr::Star => Ok(Value::Word(address)),
      UnaryExpr::Value(value) => Ok(*value),
      UnaryExpr::Name(name) => {
        match names.get(name) {
          Some(value) => Ok(*value),
          None => Err(Error::UnknownName(name.clone())),
        }
      },
    }
  }
}
impl Expression {
  pub fn resolve(&self, address: u16, names: &HashMap<String, Value>) -> Result<Value, Error> {
    match self {
      Expression::Unary(unary) => Ok(unary.resolve(address, names)?),
      Expression::High(unary) => {
        match unary.resolve(address, names)? {
          Value::Byte(_) => Err(Error::InvalidType(error::Type::Word, error::Type::Byte)),
          Value::Word(word) => Ok(Value::Byte((word >> 8) as u8)),
        }
      },
      Expression::Low(unary) => {
        match unary.resolve(address, names)? {
          Value::Byte(_) => Err(Error::InvalidType(error::Type::Word, error::Type::Byte)),
          Value::Word(word) => Ok(Value::Byte(word as u8)),
        }
      },
      Expression::Add(lhs, rhs) => {
        match (lhs.resolve(address, names)?, rhs.resolve(address, names)?) {
          (Value::Word(lhs), Value::Word(rhs)) => Ok(Value::Word(lhs + rhs)),
          (Value::Word(lhs), Value::Byte(rhs)) => Ok(Value::Word(lhs + (rhs as u16))),
          (Value::Byte(lhs), Value::Word(rhs)) => Ok(Value::Word((lhs as u16) + rhs)),
          (Value::Byte(lhs), Value::Byte(rhs)) => Ok(Value::Byte(lhs + rhs)),
        }
      },
      Expression::Sub(lhs, rhs) => {
        match (lhs.resolve(address, names)?, rhs.resolve(address, names)?) {
          (Value::Word(lhs), Value::Word(rhs)) => Ok(Value::Word(lhs - rhs)),
          (Value::Word(lhs), Value::Byte(rhs)) => Ok(Value::Word(lhs - (rhs as u16))),
          (Value::Byte(lhs), Value::Word(rhs)) => Ok(Value::Word((lhs as u16) - rhs)),
          (Value::Byte(lhs), Value::Byte(rhs)) => Ok(Value::Byte(lhs - rhs)),
        }
      },
    }
  }

  pub fn resolve_word(&self, address: u16, names: &HashMap<String, Value>) -> Result<u16, Error> {
    match self.resolve(address, names)? {
      Value::Word(word) => Ok(word),
      Value::Byte(_) => Err(Error::InvalidType(error::Type::Word, error::Type::Byte)),
    }
  }

  pub fn resolve_byte(&self, address: u16, names: &HashMap<String, Value>) -> Result<u8, Error> {
    match self.resolve(address, names)? {
      Value::Byte(byte) => Ok(byte),
      Value::Word(_) => Err(Error::InvalidType(error::Type::Byte, error::Type::Word)),
    }
  }
}

impl Register {
  pub fn assemble(&self, vec: &mut Vec<u8>, op: u8, offsets: [i8; 4]) -> Result<(), Error> {
    let mut push = |offset| {
      if offset >= 0 {
        vec.push(op + (offset as u8));
        Ok(())
      } else {
        Err(Error::InvalidOperation(error::Source::Register(self.clone()), error::Reason::Disabled))
      }
    };

    match self {
      Register::A => push(offsets[0]),
      Register::B => push(offsets[1]),
      Register::X => push(offsets[2]),
      Register::Y => push(offsets[3]),
    }
  }
}

impl Address {
  pub fn resolve(&mut self, address: u16, names: &HashMap<String, Value>) -> Result<(), Error> {
    match self {
      Address::Direct(expr) |
      Address::Indirect(expr) |
      Address::Indexed(expr, _) |
      Address::IndirectIndexed(expr, _) => {
        *expr = Expression::Unary(UnaryExpr::Value(Value::Word(expr.resolve_word(address, names)?)));
      },
    }
    Ok(())
  }

  pub fn assemble(&self, vec: &mut Vec<u8>, op: u8, offsets: [i8; 6]) -> Result<(), Error> {
    use self::Address::*;
    use self::Register::{X, Y};
    use self::Expression::Unary;
    use self::UnaryExpr::Value;
    use self::Value::Word;

    let mut push = |offset, a| {
      if offset >= 0 {
        vec.push(op + (offset as u8));
        vec.extend_from_slice(&to_bytes(a));
        Ok(())
      } else {
        Err(Error::InvalidOperation(error::Source::Address(self.clone()), error::Reason::Disabled))
      }
    };

    match self {
      Direct(Unary(Value(Word(a))))             => push(offsets[0], *a),
      Indexed(Unary(Value(Word(a))), X)         => push(offsets[1], *a),
      Indexed(Unary(Value(Word(a))), Y)         => push(offsets[2], *a),
      Indirect(Unary(Value(Word(a))))           => push(offsets[3], *a),
      IndirectIndexed(Unary(Value(Word(a))), X) => push(offsets[4], *a),
      IndirectIndexed(Unary(Value(Word(a))), Y) => push(offsets[5], *a),
      _ => Err(Error::InvalidOperation(error::Source::Address(self.clone()), error::Reason::Invalid)),
    }
  }
}

impl Argument {
  pub fn len(&self) -> u16 {
    match self {
      Argument::Register(_) => 0,
      Argument::Byte(_) => 1,
      Argument::Address(_) => 2,
    }
  }

  pub fn resolve(&mut self, address: u16, names: &HashMap<String, Value>) -> Result<(), Error> {
    match self {
      Argument::Address(a) => a.resolve(address, names),
      Argument::Byte(expr) => {
        *expr = Expression::Unary(UnaryExpr::Value(Value::Byte(expr.resolve_byte(address, names)?)));
        Ok(())
      },
      _ => Ok(()),
    }
  }
}


impl Op {
  pub fn len(&self) -> u16 {
    use self::Op::*;

    match self {
      Nop | Hlt | Brk | Int | Ret | RetI | Set(_, _) |
      Not(_) | Neg(_) | Inc(_) | Dec(_) |
      Rr(_) | RrC(_) | Rl(_) | RlC(_) |
      Pop(_) | Push(_) => 1,

      Call(_) | Jmp(_, _) => 3,

      Add(_, a) | AddC(_, a) | Sub(_, a) | SubC(_, a) |
      And(_, a) | Or(_, a) | Xor(_, a) | Cmp(_, a) => 1 + a.len(),
      Ld(d, s) => 1 + d.len() + s.len(),
    }
  }

  pub fn resolve(&mut self, address: u16, names: &HashMap<String, Value>) -> Result<(), Error> {
    use self::Op::*;

    match self {
      Jmp(_, a) | Call(a) => Ok(a.resolve(address, names)?),
      Add(_, a) | AddC(_, a) | Sub(_, a) | SubC(_, a) |
      And(_, a) | Or(_, a) | Xor(_, a) | Cmp(_, a) => Ok(a.resolve(address, names)?),
      Ld(d, s) => {
        d.resolve(address, names)?;
        s.resolve(address, names)?;
        Ok(())
      },
      _ => Ok(()),
    }
  }

  pub fn assemble(&self, vec: &mut Vec<u8>) -> Result<(), Error> {
    use self::Op::*;
    use self::Register::*;
    use self::Argument::*;

    macro_rules! Byte (
      ($i:ident) => (Argument::Byte(Expression::Unary(UnaryExpr::Value(Value::Byte($i)))))
    );

    match self {
      Nop => vec.push(0x00),
      Hlt => vec.push(0x01),
      Brk => vec.push(0x02),
      Int => vec.push(0x03),
      Set(Flag::C, false) => vec.push(0x04),
      Set(Flag::C,  true) => vec.push(0x05),
      Set(Flag::I, false) => vec.push(0x06),
      Set(Flag::I,  true) => vec.push(0x07),
      Call(a) => a.assemble(vec, 0x08, [0, -1, -1, 1, 2, 3])?,
      Ret  => vec.push(0x0C),
      RetI => vec.push(0x0D),
      Jmp(None, a) => a.assemble(vec, 0x0E, [0, -1, -1, -1, -1, -1])?,
      // 0x0F UNDEFINED
      Jmp(Some((Flag::Z, false)), a) => a.assemble(vec, 0x10, [0, -1, -1, -1, -1, -1])?,
      Jmp(Some((Flag::Z,  true)), a) => a.assemble(vec, 0x11, [0, -1, -1, -1, -1, -1])?,
      Jmp(Some((Flag::C, false)), a) => a.assemble(vec, 0x12, [0, -1, -1, -1, -1, -1])?,
      Jmp(Some((Flag::C,  true)), a) => a.assemble(vec, 0x13, [0, -1, -1, -1, -1, -1])?,
      Jmp(Some((Flag::V, false)), a) => a.assemble(vec, 0x14, [0, -1, -1, -1, -1, -1])?,
      Jmp(Some((Flag::V,  true)), a) => a.assemble(vec, 0x15, [0, -1, -1, -1, -1, -1])?,
      Jmp(Some((Flag::S, false)), a) => a.assemble(vec, 0x16, [0, -1, -1, -1, -1, -1])?,
      Jmp(Some((Flag::S,  true)), a) => a.assemble(vec, 0x17, [0, -1, -1, -1, -1, -1])?,
      Inc(X)           => vec.push(0x18),
      Dec(X)           => vec.push(0x19),
      Add(X, Byte!(b)) => { vec.push(0x1A); vec.push(*b); }
      Sub(X, Byte!(b)) => { vec.push(0x1B); vec.push(*b); }
      Inc(Y)           => vec.push(0x1C),
      Dec(Y)           => vec.push(0x1D),
      Add(Y, Byte!(b)) => { vec.push(0x1E); vec.push(*b); }
      Sub(Y, Byte!(b)) => { vec.push(0x1F); vec.push(*b); }
      Cmp(X, Register(A)) => vec.push(0x20),
      Cmp(X, Register(B)) => vec.push(0x21),
      Cmp(X, Byte!(b))    => { vec.push(0x22); vec.push(*b); },
      Cmp(X, Register(Y)) => vec.push(0x23),
      Cmp(X, Address(a))  => a.assemble(vec, 0x24, [0, -1, 1, 2, -1, 3])?,
      Cmp(Y, Register(A)) => vec.push(0x28),
      Cmp(Y, Register(B)) => vec.push(0x29),
      Cmp(Y, Register(X)) => vec.push(0x2A),
      Cmp(Y, Byte!(b))    => { vec.push(0x2B); vec.push(*b); }
      Cmp(Y, Address(a))  => a.assemble(vec, 0x2C, [0, 1, -1, 2, 3, -1])?,
      Not(r) => r.assemble(vec, 0x30, [0, 8, -1, -1])?,
      Neg(r) => r.assemble(vec, 0x31, [0, 8, -1, -1])?,
      Inc(r) => r.assemble(vec, 0x32, [0, 8, -1, -1])?,
      Dec(r) => r.assemble(vec, 0x33, [0, 8, -1, -1])?,
      Rr(r)  => r.assemble(vec, 0x34, [0, 8, -1, -1])?,
      RrC(r) => r.assemble(vec, 0x35, [0, 8, -1, -1])?,
      Rl(r)  => r.assemble(vec, 0x36, [0, 8, -1, -1])?,
      RlC(r) => r.assemble(vec, 0x37, [0, 8, -1, -1])?,
      Add( r, Byte!(b))    => { r.assemble(vec, 0x40, [32, 96, -1, -1])?; vec.push(*b); },
      Add( A, Register(B)) => vec.push(0x40),
      Add( A, Address(a))  => a.assemble(vec, 0x40, [8, 16, 24, 40, 48, 56])?,
      Add( B, Register(A)) => vec.push(0x80),
      Add( B, Address(a))  => a.assemble(vec, 0x80, [8, 16, 24, 40, 48, 56])?,
      AddC(r, Byte!(b))    => { r.assemble(vec, 0x41, [32, 96, -1, -1])?; vec.push(*b); },
      AddC(A, Register(B)) => vec.push(0x41),
      AddC(A, Address(a))  => a.assemble(vec, 0x41, [8, 16, 24, 40, 48, 56])?,
      AddC(B, Register(A)) => vec.push(0x81),
      AddC(B, Address(a))  => a.assemble(vec, 0x81, [8, 16, 24, 40, 48, 56])?,
      Sub( r, Byte!(b))    => { r.assemble(vec, 0x42, [32, 96, -1, -1])?; vec.push(*b); },
      Sub( A, Register(B)) => vec.push(0x42),
      Sub( A, Address(a))  => a.assemble(vec, 0x42, [8, 16, 24, 40, 48, 56])?,
      Sub( B, Register(A)) => vec.push(0x82),
      Sub( B, Address(a))  => a.assemble(vec, 0x82, [8, 16, 24, 40, 48, 56])?,
      SubC(r, Byte!(b))    => { r.assemble(vec, 0x43, [32, 96, -1, -1])?; vec.push(*b); },
      SubC(A, Register(B)) => vec.push(0x43),
      SubC(A, Address(a))  => a.assemble(vec, 0x43, [8, 16, 24, 40, 48, 56])?,
      SubC(B, Register(A)) => vec.push(0x83),
      SubC(B, Address(a))  => a.assemble(vec, 0x83, [8, 16, 24, 40, 48, 56])?,
      And( r, Byte!(b))    => { r.assemble(vec, 0x44, [32, 96, -1, -1])?; vec.push(*b); },
      And( A, Register(B)) => vec.push(0x44),
      And( A, Address(a))  => a.assemble(vec, 0x44, [8, 16, 24, 40, 48, 56])?,
      And( B, Register(A)) => vec.push(0x84),
      And( B, Address(a))  => a.assemble(vec, 0x84, [8, 16, 24, 40, 48, 56])?,
      Or(  r, Byte!(b))    => { r.assemble(vec, 0x45, [32, 96, -1, -1])?; vec.push(*b); },
      Or(  A, Register(B)) => vec.push(0x45),
      Or(  A, Address(a))  => a.assemble(vec, 0x45, [8, 16, 24, 40, 48, 56])?,
      Or(  B, Register(A)) => vec.push(0x85),
      Or(  B, Address(a))  => a.assemble(vec, 0x85, [8, 16, 24, 40, 48, 56])?,
      Xor( r, Byte!(b))    => { r.assemble(vec, 0x46, [32, 96, -1, -1])?; vec.push(*b); },
      Xor( A, Register(B)) => vec.push(0x46),
      Xor( A, Address(a))  => a.assemble(vec, 0x46, [8, 16, 24, 40, 48, 56])?,
      Xor( B, Register(A)) => vec.push(0x86),
      Xor( B, Address(a))  => a.assemble(vec, 0x86, [8, 16, 24, 40, 48, 56])?,
      Cmp( r, Byte!(b))    => { r.assemble(vec, 0x47, [32, 96, -1, -1])?; vec.push(*b); },
      Cmp( A, Register(B)) => vec.push(0x47),
      Cmp( A, Address(a))  => a.assemble(vec, 0x47, [8, 16, 24, 40, 48, 56])?,
      Cmp( B, Register(A)) => vec.push(0x87),
      Cmp( B, Address(a))  => a.assemble(vec, 0x87, [8, 16, 24, 40, 48, 56])?,
      Ld(Register(r), Byte!(b))    => { r.assemble(vec, 0xC0, [0, 5, 10, 15])?; vec.push(*b) },
      Ld(Register(A), Register(r)) => r.assemble(vec, 0xC0, [-1, 1, 2, 3])?,
      Ld(Register(B), Register(r)) => r.assemble(vec, 0xC4, [0, -1, 2, 3])?,
      Ld(Register(X), Register(r)) => r.assemble(vec, 0xC8, [0, 1, -1, 3])?,
      Ld(Register(Y), Register(r)) => r.assemble(vec, 0xCC, [0, 1, 2, -1])?,
      Ld(Register(X), Address(a))  => a.assemble(vec, 0xD0, [0, -1, 1, 2, -1, 3])?,
      Ld(Register(Y), Address(a))  => a.assemble(vec, 0xD4, [0, 1, -1, 2, 3, -1])?,
      Ld(Address(a), Register(X))  => a.assemble(vec, 0xD8, [0, -1, 1, 2, -1, 3])?,
      Ld(Address(a), Register(Y))  => a.assemble(vec, 0xDC, [0, 1, -1, 2, 3, -1])?,
      Pop(r)                       => r.assemble(vec, 0xE0, [0, 4, 8, 12])?,
      Ld(Register(A), Address(a))  => a.assemble(vec, 0xE0, [1, 2, 3, 5, 6, 7])?,
      Ld(Register(B), Address(a))  => a.assemble(vec, 0xE8, [1, 2, 3, 5, 6, 7])?,
      Push(r)                      => r.assemble(vec, 0xF0, [0, 4, 8, 12])?,
      Ld(Address(a), Register(A))  => a.assemble(vec, 0xF0, [1, 2, 3, 5, 6, 7])?,
      Ld(Address(a), Register(B))  => a.assemble(vec, 0xF8, [1, 2, 3, 5, 6, 7])?,
      _ => return Err(Error::InvalidOperation(error::Source::Operation(self.clone()), error::Reason::Invalid)),
    }

    Ok(())
  }
}
