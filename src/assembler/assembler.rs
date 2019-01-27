
use std::collections::HashMap;

use crate::math::*;
use super::error::{self, Error};
use super::tokens::*;


impl Operand {
  pub fn word(&self, address: u16, names: &HashMap<String, u16>) -> Result<u16, Error> {
    match self {
      Operand::Star => Ok(address),
      Operand::Number(number) => Ok(*number),
      Operand::Name(name) => {
        match names.get(name) {
          Some(value) => Ok(*value),
          None => Err(Error::UnknownName(name.clone())),
        }
      },
    }
  }

  pub fn byte(&self, address: u16, names: &HashMap<String, u16>) -> Result<u8, Error> {
    if let Operand::Star = self {
      return Err(Error::InvalidType(error::Type::Word, error::Type::Byte))
    }
    let word = self.word(address, names)?;
    if ((word as i16) < -128) || (word > 255) {
      return Err(Error::InvalidType(error::Type::Word, error::Type::Byte))
    }
    Ok(word as u8)
  }
}
impl Expression {
  pub fn word(&self, address: u16, names: &HashMap<String, u16>) -> Result<u16, Error> {
    match self {
      Expression::Unary(unary)  => Ok(unary.word(address, names)?),
      Expression::High(_)       => Err(Error::InvalidType(error::Type::Byte, error::Type::Word)),
      Expression::Low(_)        => Err(Error::InvalidType(error::Type::Byte, error::Type::Word)),
      Expression::Add(lhs, rhs) => Ok(lhs.word(address, names)? + rhs.word(address, names)?),
      Expression::Sub(lhs, rhs) => Ok(lhs.word(address, names)? - rhs.word(address, names)?),
    }
  }

  pub fn byte(&self, address: u16, names: &HashMap<String, u16>) -> Result<u8, Error> {
    match self {
      Expression::Unary(unary)  => Ok(unary.byte(address, names)?),
      Expression::High(unary)   => Ok((unary.word(address, names)? >> 8) as u8),
      Expression::Low(unary)    => Ok(unary.word(address, names)? as u8),
      Expression::Add(lhs, rhs) => Ok(lhs.byte(address, names)? + rhs.byte(address, names)?),
      Expression::Sub(lhs, rhs) => Ok(lhs.byte(address, names)? - rhs.byte(address, names)?),
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
  pub fn assemble(&self, vec: &mut Vec<u8>, op: u8, offsets: [i8; 6], address: u16, names: &HashMap<String, u16>) -> Result<(), Error> {
    use self::Address::*;
    use self::Register::{X, Y};

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
      Direct(expr)             => push(offsets[0], expr.word(address, names)?),
      Indexed(expr, X)         => push(offsets[1], expr.word(address, names)?),
      Indexed(expr, Y)         => push(offsets[2], expr.word(address, names)?),
      Indirect(expr)           => push(offsets[3], expr.word(address, names)?),
      IndirectIndexed(expr, X) => push(offsets[4], expr.word(address, names)?),
      IndirectIndexed(expr, Y) => push(offsets[5], expr.word(address, names)?),
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

  pub fn assemble(&self, vec: &mut Vec<u8>, address: u16, names: &HashMap<String, u16>) -> Result<(), Error> {
    use self::Op::*;
    use self::Register::*;
    use self::Argument::*;

    match self {
      Nop => vec.push(0x00),
      Hlt => vec.push(0x01),
      Brk => vec.push(0x02),
      Int => vec.push(0x03),
      Set(Flag::C, false) => vec.push(0x04),
      Set(Flag::C,  true) => vec.push(0x05),
      Set(Flag::I, false) => vec.push(0x06),
      Set(Flag::I,  true) => vec.push(0x07),
      Call(a) => a.assemble(vec, 0x08, [0, -1, -1, 1, 2, 3], address, names)?,
      Ret  => vec.push(0x0C),
      RetI => vec.push(0x0D),
      Jmp(None, a) => a.assemble(vec, 0x0E, [0, -1, -1, -1, -1, -1], address, names)?,
      // 0x0F UNDEFINED
      Jmp(Some((Flag::Z, false)), a) => a.assemble(vec, 0x10, [0, -1, -1, -1, -1, -1], address, names)?,
      Jmp(Some((Flag::Z,  true)), a) => a.assemble(vec, 0x11, [0, -1, -1, -1, -1, -1], address, names)?,
      Jmp(Some((Flag::C, false)), a) => a.assemble(vec, 0x12, [0, -1, -1, -1, -1, -1], address, names)?,
      Jmp(Some((Flag::C,  true)), a) => a.assemble(vec, 0x13, [0, -1, -1, -1, -1, -1], address, names)?,
      Jmp(Some((Flag::V, false)), a) => a.assemble(vec, 0x14, [0, -1, -1, -1, -1, -1], address, names)?,
      Jmp(Some((Flag::V,  true)), a) => a.assemble(vec, 0x15, [0, -1, -1, -1, -1, -1], address, names)?,
      Jmp(Some((Flag::S, false)), a) => a.assemble(vec, 0x16, [0, -1, -1, -1, -1, -1], address, names)?,
      Jmp(Some((Flag::S,  true)), a) => a.assemble(vec, 0x17, [0, -1, -1, -1, -1, -1], address, names)?,
      Inc(X)          => vec.push(0x18),
      Dec(X)          => vec.push(0x19),
      Add(X, Byte(b)) => { vec.push(0x1A); vec.push(b.byte(address, names)?); }
      Sub(X, Byte(b)) => { vec.push(0x1B); vec.push(b.byte(address, names)?); }
      Inc(Y)          => vec.push(0x1C),
      Dec(Y)          => vec.push(0x1D),
      Add(Y, Byte(b)) => { vec.push(0x1E); vec.push(b.byte(address, names)?); }
      Sub(Y, Byte(b)) => { vec.push(0x1F); vec.push(b.byte(address, names)?); }
      Cmp(X, Register(A)) => vec.push(0x20),
      Cmp(X, Register(B)) => vec.push(0x21),
      Cmp(X, Byte(b))     => { vec.push(0x22); vec.push(b.byte(address, names)?); },
      Cmp(X, Register(Y)) => vec.push(0x23),
      Cmp(X, Address(a))  => a.assemble(vec, 0x24, [0, -1, 1, 2, -1, 3], address, names)?,
      Cmp(Y, Register(A)) => vec.push(0x28),
      Cmp(Y, Register(B)) => vec.push(0x29),
      Cmp(Y, Register(X)) => vec.push(0x2A),
      Cmp(Y, Byte(b))     => { vec.push(0x2B); vec.push(b.byte(address, names)?); }
      Cmp(Y, Address(a))  => a.assemble(vec, 0x2C, [0, 1, -1, 2, 3, -1], address, names)?,
      Not(r) => r.assemble(vec, 0x30, [0, 8, -1, -1])?,
      Neg(r) => r.assemble(vec, 0x31, [0, 8, -1, -1])?,
      Inc(r) => r.assemble(vec, 0x32, [0, 8, -1, -1])?,
      Dec(r) => r.assemble(vec, 0x33, [0, 8, -1, -1])?,
      Rr(r)  => r.assemble(vec, 0x34, [0, 8, -1, -1])?,
      RrC(r) => r.assemble(vec, 0x35, [0, 8, -1, -1])?,
      Rl(r)  => r.assemble(vec, 0x36, [0, 8, -1, -1])?,
      RlC(r) => r.assemble(vec, 0x37, [0, 8, -1, -1])?,
      Add( r, Byte(b))     => { r.assemble(vec, 0x40, [32, 96, -1, -1])?; vec.push(b.byte(address, names)?); },
      Add( A, Register(B)) => vec.push(0x40),
      Add( A, Address(a))  => a.assemble(vec, 0x40, [8, 16, 24, 40, 48, 56], address, names)?,
      Add( B, Register(A)) => vec.push(0x80),
      Add( B, Address(a))  => a.assemble(vec, 0x80, [8, 16, 24, 40, 48, 56], address, names)?,
      AddC(r, Byte(b))     => { r.assemble(vec, 0x41, [32, 96, -1, -1])?; vec.push(b.byte(address, names)?); },
      AddC(A, Register(B)) => vec.push(0x41),
      AddC(A, Address(a))  => a.assemble(vec, 0x41, [8, 16, 24, 40, 48, 56], address, names)?,
      AddC(B, Register(A)) => vec.push(0x81),
      AddC(B, Address(a))  => a.assemble(vec, 0x81, [8, 16, 24, 40, 48, 56], address, names)?,
      Sub( r, Byte(b))     => { r.assemble(vec, 0x42, [32, 96, -1, -1])?; vec.push(b.byte(address, names)?); },
      Sub( A, Register(B)) => vec.push(0x42),
      Sub( A, Address(a))  => a.assemble(vec, 0x42, [8, 16, 24, 40, 48, 56], address, names)?,
      Sub( B, Register(A)) => vec.push(0x82),
      Sub( B, Address(a))  => a.assemble(vec, 0x82, [8, 16, 24, 40, 48, 56], address, names)?,
      SubC(r, Byte(b))     => { r.assemble(vec, 0x43, [32, 96, -1, -1])?; vec.push(b.byte(address, names)?); },
      SubC(A, Register(B)) => vec.push(0x43),
      SubC(A, Address(a))  => a.assemble(vec, 0x43, [8, 16, 24, 40, 48, 56], address, names)?,
      SubC(B, Register(A)) => vec.push(0x83),
      SubC(B, Address(a))  => a.assemble(vec, 0x83, [8, 16, 24, 40, 48, 56], address, names)?,
      And( r, Byte(b))     => { r.assemble(vec, 0x44, [32, 96, -1, -1])?; vec.push(b.byte(address, names)?); },
      And( A, Register(B)) => vec.push(0x44),
      And( A, Address(a))  => a.assemble(vec, 0x44, [8, 16, 24, 40, 48, 56], address, names)?,
      And( B, Register(A)) => vec.push(0x84),
      And( B, Address(a))  => a.assemble(vec, 0x84, [8, 16, 24, 40, 48, 56], address, names)?,
      Or(  r, Byte(b))     => { r.assemble(vec, 0x45, [32, 96, -1, -1])?; vec.push(b.byte(address, names)?); },
      Or(  A, Register(B)) => vec.push(0x45),
      Or(  A, Address(a))  => a.assemble(vec, 0x45, [8, 16, 24, 40, 48, 56], address, names)?,
      Or(  B, Register(A)) => vec.push(0x85),
      Or(  B, Address(a))  => a.assemble(vec, 0x85, [8, 16, 24, 40, 48, 56], address, names)?,
      Xor( r, Byte(b))     => { r.assemble(vec, 0x46, [32, 96, -1, -1])?; vec.push(b.byte(address, names)?); },
      Xor( A, Register(B)) => vec.push(0x46),
      Xor( A, Address(a))  => a.assemble(vec, 0x46, [8, 16, 24, 40, 48, 56], address, names)?,
      Xor( B, Register(A)) => vec.push(0x86),
      Xor( B, Address(a))  => a.assemble(vec, 0x86, [8, 16, 24, 40, 48, 56], address, names)?,
      Cmp( r, Byte(b))     => { r.assemble(vec, 0x47, [32, 96, -1, -1])?; vec.push(b.byte(address, names)?); },
      Cmp( A, Register(B)) => vec.push(0x47),
      Cmp( A, Address(a))  => a.assemble(vec, 0x47, [8, 16, 24, 40, 48, 56], address, names)?,
      Cmp( B, Register(A)) => vec.push(0x87),
      Cmp( B, Address(a))  => a.assemble(vec, 0x87, [8, 16, 24, 40, 48, 56], address, names)?,
      Ld(Register(r), Byte(b))     => { r.assemble(vec, 0xC0, [0, 5, 10, 15])?; vec.push(b.byte(address, names)?) },
      Ld(Register(A), Register(r)) => r.assemble(vec, 0xC0, [-1, 1, 2, 3])?,
      Ld(Register(B), Register(r)) => r.assemble(vec, 0xC4, [0, -1, 2, 3])?,
      Ld(Register(X), Register(r)) => r.assemble(vec, 0xC8, [0, 1, -1, 3])?,
      Ld(Register(Y), Register(r)) => r.assemble(vec, 0xCC, [0, 1, 2, -1])?,
      Ld(Register(X), Address(a))  => a.assemble(vec, 0xD0, [0, -1, 1, 2, -1, 3], address, names)?,
      Ld(Register(Y), Address(a))  => a.assemble(vec, 0xD4, [0, 1, -1, 2, 3, -1], address, names)?,
      Ld(Address(a), Register(X))  => a.assemble(vec, 0xD8, [0, -1, 1, 2, -1, 3], address, names)?,
      Ld(Address(a), Register(Y))  => a.assemble(vec, 0xDC, [0, 1, -1, 2, 3, -1], address, names)?,
      Pop(r)                       => r.assemble(vec, 0xE0, [0, 4, 8, 12])?,
      Ld(Register(A), Address(a))  => a.assemble(vec, 0xE0, [1, 2, 3, 5, 6, 7], address, names)?,
      Ld(Register(B), Address(a))  => a.assemble(vec, 0xE8, [1, 2, 3, 5, 6, 7], address, names)?,
      Push(r)                      => r.assemble(vec, 0xF0, [0, 4, 8, 12])?,
      Ld(Address(a), Register(A))  => a.assemble(vec, 0xF0, [1, 2, 3, 5, 6, 7], address, names)?,
      Ld(Address(a), Register(B))  => a.assemble(vec, 0xF8, [1, 2, 3, 5, 6, 7], address, names)?,
      _ => return Err(Error::InvalidOperation(error::Source::Operation(self.clone()), error::Reason::Invalid)),
    }

    Ok(())
  }
}
