
use std::collections::HashMap;

use nom::types::CompleteStr;
use nom::{
  digit,
  hex_digit,
  alphanumeric1,
};


fn to_bytes(a: u16) -> [u8; 2] {
  [(a >> 8) as u8, a as u8]
}

#[derive(Debug)]
pub enum Flag { Z, C, V, S, I }

#[derive(Debug)]
pub enum Register { A, B, X, Y }
impl Register {
  pub fn length(&self) -> Option<u16> {
    Some(0)
  }

  pub fn assemble(&self, vec: &mut Vec<u8>, op: u8, offsets: [i8; 4]) {
    use self::Register::*;

    let mut push = |offset| {
      if offset >= 0 {
        vec.push(op + (offset as u8));
      } else {
        panic!("Attempted to assemble invalid operation.");
      }
    };

    match self {
      A => push(offsets[0]),
      B => push(offsets[1]),
      X => push(offsets[2]),
      Y => push(offsets[3]),
    }
  }
}

#[derive(Debug)]
pub enum Labeled {
  Const(u16),
  Label(String),
}

#[derive(Debug)]
pub enum Address {
  Offset(i8),
  Direct(Labeled),
  Indexed(Labeled, Register),
  Indirect(Labeled),
  IndirectIndexed(Labeled, Register),
}
impl Address {
  pub fn length(&self) -> Option<u16> {
    if let Address::Offset(_) = self {
      return Some(1)
    }

    let label = match self {
      Address::Direct(l) |
      Address::Indirect(l) |
      Address::Indexed(l, _) |
      Address::IndirectIndexed(l, _) => l,
      _ => return None,
    };

    match label {
      Labeled::Label(_) => None,
      Labeled::Const(_) => Some(2),
    }
  }

  pub fn resolve(&mut self, allow_offset: bool, address: u16, labels: &HashMap<String, u16>) {
    match self {
      Address::Direct(Labeled::Label(label)) => {
        match labels.get(label) {
          Some(l_addr) => {
            let offset = (*l_addr as i32) - (address as i32);
            if allow_offset && (-127 <= offset) && (offset <= 125) {
              if offset < 0 {
                // PC is at the beginning of the next instruction.
                *self = Address::Offset((offset - 1) as i8);
              } else {
                *self = Address::Offset((offset + 2) as i8);
              }
            } else {
              *self = Address::Direct(Labeled::Const(*l_addr));
            }
          },
          None => (),
        }
      },
      Address::Indirect(a) |
      Address::Indexed(a, _) |
      Address::IndirectIndexed(a, _) => {
        if let Labeled::Label(label) = a {
          match labels.get(label) {
            Some(l_addr) => *a = Labeled::Const(*l_addr),
            None => (),
          }
        }
      },
      _ => (),
    }
  }

  pub fn assemble(&self, vec: &mut Vec<u8>, op: u8, offsets: [i8; 7]) {
    use self::Labeled::Const;
    use self::Address::*;
    use self::Register::{X, Y};

    let mut push = |offset, a| {
      if offset >= 0 {
        vec.push(op + (offset as u8));
        vec.extend_from_slice(&to_bytes(a))
      } else {
        panic!("Attempted to assemble invalid operation.");
      }
    };

    println!("Assembling {:?}", self);
    match self {
      Offset(o) => {
        vec.push(op + (offsets[0] as u8));
        vec.push(*o as u8);
      },
      Direct(Const(a))             => push(offsets[1], *a),
      Indexed(Const(a), X)         => push(offsets[2], *a),
      Indexed(Const(a), Y)         => push(offsets[3], *a),
      Indirect(Const(a))           => push(offsets[4], *a),
      IndirectIndexed(Const(a), X) => push(offsets[5], *a),
      IndirectIndexed(Const(a), Y) => push(offsets[6], *a),
      _ => panic!("Attempted to assemble invalid operation."),
    }
  }
}

#[derive(Debug)]
pub enum Argument {
  Byte(u8),
  Register(Register),
  Address(Address),
}
impl Argument {
  pub fn length(&self) -> Option<u16> {
    match self {
      Argument::Byte(_) => Some(1),
      Argument::Register(_) => Some(0),
      Argument::Address(a) => a.length(),
    }
  }

  pub fn resolve(&mut self, allow_offset: bool, address: u16, labels: &HashMap<String, u16>) {
    match self {
      Argument::Address(a) => a.resolve(allow_offset, address, labels),
      _ => (),
    }
  }
}

#[derive(Debug)]
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
impl Op {
  pub fn length(&self) -> Option<u16> {
    use self::Op::*;

    let offset = |length| {
      match length {
        Some(l) => Some(1 + l),
        None => None,
      }
    };
    let offset2 = |l1, l2| {
      match (l1, l2) {
        (Some(a), Some(b)) => Some(1 + a + b),
        _ => None,
      }
    };

    match self {
      Nop | Hlt | Brk | Int | Ret | RetI => Some(1),
      Set(_, _) => Some(1),
      Call(a) => offset(a.length()),
      Jmp(Some(_), _) => Some(2),
      Jmp(None, a) => offset(a.length()),
      Not(r) | Neg(r) | Inc(r) | Dec(r) | Rr(r) | RrC(r) | Rl(r) | RlC(r) | Pop(r) | Push(r) => offset(r.length()),
      Add(r, a) | AddC(r, a) | Sub(r, a) | SubC(r, a) | And(r, a) | Or(r, a) | Xor(r, a) | Cmp(r, a) => offset2(r.length(), a.length()),
      Ld(d, s) => offset2(d.length(), s.length()),
    }
  }

  pub fn resolve(&mut self, address: u16, labels: &HashMap<String, u16>) {
    use self::Op::*;
    use self::Address::*;
    use self::Labeled::*;

    match self {
      Jmp(_, a) => a.resolve(true, address, labels),
      Call(a) => a.resolve(false, address, labels),
      Add(_, a) | AddC(_, a) | Sub(_, a) | SubC(_, a) |
      And(_, a) | Or(_, a) | Xor(_, a) | Cmp(_, a) => a.resolve(false, address, labels),
      Ld(d, s) => {
        d.resolve(false, address, labels);
        s.resolve(false, address, labels);
      },
      _ => (),
    }
  }

  pub fn assemble(&self, vec: &mut Vec<u8>) {
    use self::Labeled::Const;
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
      Call(a) => a.assemble(vec, 0x08, [-1, 0, -1, -1, 1, 2, 3]),
      Ret  => vec.push(0x0C),
      RetI => vec.push(0x0D),
      Jmp(None, a) => a.assemble(vec, 0x0E, [1, 0, -1, -1, -1, -1, -1]),
      Jmp(Some((Flag::Z, false)), a) => a.assemble(vec, 0x10, [0, -1, -1, -1, -1, -1, -1]),
      Jmp(Some((Flag::Z,  true)), a) => a.assemble(vec, 0x11, [0, -1, -1, -1, -1, -1, -1]),
      Jmp(Some((Flag::C, false)), a) => a.assemble(vec, 0x12, [0, -1, -1, -1, -1, -1, -1]),
      Jmp(Some((Flag::C,  true)), a) => a.assemble(vec, 0x13, [0, -1, -1, -1, -1, -1, -1]),
      Jmp(Some((Flag::V, false)), a) => a.assemble(vec, 0x14, [0, -1, -1, -1, -1, -1, -1]),
      Jmp(Some((Flag::V,  true)), a) => a.assemble(vec, 0x15, [0, -1, -1, -1, -1, -1, -1]),
      Jmp(Some((Flag::S, false)), a) => a.assemble(vec, 0x16, [0, -1, -1, -1, -1, -1, -1]),
      Jmp(Some((Flag::S,  true)), a) => a.assemble(vec, 0x17, [0, -1, -1, -1, -1, -1, -1]),
      Inc(X)          => vec.push(0x18),
      Dec(X)          => vec.push(0x19),
      Add(X, Byte(b)) => { vec.push(0x1A); vec.push(*b); }
      Sub(X, Byte(b)) => { vec.push(0x1B); vec.push(*b); }
      Inc(Y)          => vec.push(0x1C),
      Dec(Y)          => vec.push(0x1D),
      Add(Y, Byte(b)) => { vec.push(0x1E); vec.push(*b); }
      Sub(Y, Byte(b)) => { vec.push(0x1F); vec.push(*b); }
      Cmp(X, Register(A)) => vec.push(0x20),
      Cmp(X, Register(B)) => vec.push(0x21),
      Cmp(X, Byte(b))     => { vec.push(0x22); vec.push(*b); },
      Cmp(X, Register(Y)) => vec.push(0x23),
      Cmp(X, Address(a))  => a.assemble(vec, 0x24, [-1, 0, -1, 1, 2, -1, 3]),
      Cmp(Y, Register(A)) => vec.push(0x28),
      Cmp(Y, Register(B)) => vec.push(0x29),
      Cmp(Y, Register(X)) => vec.push(0x2A),
      Cmp(Y, Byte(b))     => { vec.push(0x2B); vec.push(*b); }
      Cmp(Y, Address(a))  => a.assemble(vec, 0x2C, [-1, 0, 1, -1, 2, 3, -1]),
      Not(r) => r.assemble(vec, 0x30, [0, 8, -1, -1]),
      Neg(r) => r.assemble(vec, 0x31, [0, 8, -1, -1]),
      Inc(r) => r.assemble(vec, 0x32, [0, 8, -1, -1]),
      Dec(r) => r.assemble(vec, 0x33, [0, 8, -1, -1]),
      Rr(r)  => r.assemble(vec, 0x34, [0, 8, -1, -1]),
      RrC(r) => r.assemble(vec, 0x35, [0, 8, -1, -1]),
      Rl(r)  => r.assemble(vec, 0x36, [0, 8, -1, -1]),
      RlC(r) => r.assemble(vec, 0x37, [0, 8, -1, -1]),
      Add( r, Byte(b))     => { r.assemble(vec, 0x40, [32, 96, -1, -1]); vec.push(*b); },
      Add( A, Register(B)) => vec.push(0x40),
      Add( A, Address(a))  => a.assemble(vec, 0x40, [-1, 8, 16, 24, 40, 48, 56]),
      Add( B, Register(A)) => vec.push(0x80),
      Add( B, Address(a))  => a.assemble(vec, 0x80, [-1, 8, 16, 24, 40, 48, 56]),
      AddC(r, Byte(b))     => { r.assemble(vec, 0x41, [32, 96, -1, -1]); vec.push(*b); },
      AddC(A, Register(B)) => vec.push(0x41),
      AddC(A, Address(a))  => a.assemble(vec, 0x41, [-1, 8, 16, 24, 40, 48, 56]),
      AddC(B, Register(A)) => vec.push(0x81),
      AddC(B, Address(a))  => a.assemble(vec, 0x81, [-1, 8, 16, 24, 40, 48, 56]),
      Sub( r, Byte(b))     => { r.assemble(vec, 0x42, [32, 96, -1, -1]); vec.push(*b); },
      Sub( A, Register(B)) => vec.push(0x42),
      Sub( A, Address(a))  => a.assemble(vec, 0x42, [-1, 8, 16, 24, 40, 48, 56]),
      Sub( B, Register(A)) => vec.push(0x82),
      Sub( B, Address(a))  => a.assemble(vec, 0x82, [-1, 8, 16, 24, 40, 48, 56]),
      SubC(r, Byte(b))     => { r.assemble(vec, 0x43, [32, 96, -1, -1]); vec.push(*b); },
      SubC(A, Register(B)) => vec.push(0x43),
      SubC(A, Address(a))  => a.assemble(vec, 0x43, [-1, 8, 16, 24, 40, 48, 56]),
      SubC(B, Register(A)) => vec.push(0x83),
      SubC(B, Address(a))  => a.assemble(vec, 0x83, [-1, 8, 16, 24, 40, 48, 56]),
      And( r, Byte(b))     => { r.assemble(vec, 0x44, [32, 96, -1, -1]); vec.push(*b); },
      And( A, Register(B)) => vec.push(0x44),
      And( A, Address(a))  => a.assemble(vec, 0x44, [-1, 8, 16, 24, 40, 48, 56]),
      And( B, Register(A)) => vec.push(0x84),
      And( B, Address(a))  => a.assemble(vec, 0x84, [-1, 8, 16, 24, 40, 48, 56]),
      Or(  r, Byte(b))     => { r.assemble(vec, 0x45, [32, 96, -1, -1]); vec.push(*b); },
      Or(  A, Register(B)) => vec.push(0x45),
      Or(  A, Address(a))  => a.assemble(vec, 0x45, [-1, 8, 16, 24, 40, 48, 56]),
      Or(  B, Register(A)) => vec.push(0x85),
      Or(  B, Address(a))  => a.assemble(vec, 0x85, [-1, 8, 16, 24, 40, 48, 56]),
      Xor( r, Byte(b))     => { r.assemble(vec, 0x46, [32, 96, -1, -1]); vec.push(*b); },
      Xor( A, Register(B)) => vec.push(0x46),
      Xor( A, Address(a))  => a.assemble(vec, 0x46, [-1, 8, 16, 24, 40, 48, 56]),
      Xor( B, Register(A)) => vec.push(0x86),
      Xor( B, Address(a))  => a.assemble(vec, 0x86, [-1, 8, 16, 24, 40, 48, 56]),
      Cmp( r, Byte(b))     => { r.assemble(vec, 0x47, [32, 96, -1, -1]); vec.push(*b); },
      Cmp( A, Register(B)) => vec.push(0x47),
      Cmp( A, Address(a))  => a.assemble(vec, 0x47, [-1, 8, 16, 24, 40, 48, 56]),
      Cmp( B, Register(A)) => vec.push(0x87),
      Cmp( B, Address(a))  => a.assemble(vec, 0x87, [-1, 8, 16, 24, 40, 48, 56]),
      Ld(Register(r), Byte(b))     => { r.assemble(vec, 0xC0, [0, 5, 10, 15]); vec.push(*b) },
      Ld(Register(A), Register(r)) => r.assemble(vec, 0xC0, [-1, 1, 2, 3]),
      Ld(Register(B), Register(r)) => r.assemble(vec, 0xC4, [0, -1, 2, 3]),
      Ld(Register(X), Register(r)) => r.assemble(vec, 0xC8, [0, 1, -1, 3]),
      Ld(Register(Y), Register(r)) => r.assemble(vec, 0xCC, [0, 1, 2, -1]),
      Ld(Register(X), Address(a))  => a.assemble(vec, 0xD0, [-1, 0, -1, 1, 2, -1, 3]),
      Ld(Register(Y), Address(a))  => a.assemble(vec, 0xD4, [-1, 0, 1, -1, 2, 3, -1]),
      Ld(Address(a), Register(X))  => a.assemble(vec, 0xD8, [-1, 0, -1, 1, 2, -1, 3]),
      Ld(Address(a), Register(Y))  => a.assemble(vec, 0xDC, [-1, 0, 1, -1, 2, 3, -1]),
      Pop(r)                       => r.assemble(vec, 0xE0, [0, 4, 8, 12]),
      Ld(Register(A), Address(a))  => a.assemble(vec, 0xE0, [-1, 1, 2, 3, 5, 6, 7]),
      Ld(Register(B), Address(a))  => a.assemble(vec, 0xE8, [-1, 1, 2, 3, 5, 6, 7]),
      Push(r)                      => r.assemble(vec, 0xF0, [0, 4, 8, 12]),
      Ld(Address(a), Register(A))  => a.assemble(vec, 0xF0, [-1, 1, 2, 3, 5, 6, 7]),
      Ld(Address(a), Register(B))  => a.assemble(vec, 0xF8, [-1, 1, 2, 3, 5, 6, 7]),
      _ => panic!("Attempted to assemble invalid operation."),
    }
  }
}

#[derive(Debug)]
pub enum Token {
  Section(u16),
  Label(String),
  Op(Op),
}


named!(eat_sp(CompleteStr) -> CompleteStr, eat_separator!(&b" \t"[..]));
macro_rules! sp (
  ($i:expr, $($args:tt)*) => ({
    use nom::Convert;
    use nom::Err;
    use std::result::Result::*;

    match sep!($i, eat_sp, $($args)*) {
      Err(e) => Err(e),
      Ok((i1, o)) => {
        match (eat_sp)(i1) {
          Err(e) => Err(Err::convert(e)),
          Ok((i2,_)) => Ok((i2, o))
        }
      },
    }
  })
);

named!(comma(CompleteStr) -> char, one_of!(","));


named!(byte(CompleteStr) -> u8, alt!(
  map!(preceded!(tag!("0x"), hex_digit), |s| u8::from_str_radix(&s, 16).unwrap()) |
  map!(recognize!(pair!(opt!(one_of!("+")), digit)), |s| u8::from_str_radix(&s, 10).unwrap()) |
  map!(recognize!(pair!(one_of!("-"), digit)), |s| i8::from_str_radix(&s, 10).unwrap() as u8)
));
named!(address(CompleteStr) -> u16,
  map!(preceded!(tag!("0x"), hex_digit), |s| u16::from_str_radix(&s, 16).unwrap())
);

named!(label(CompleteStr) -> String, map!(
  recognize!(pair!(one_of!("."), alphanumeric1)),
  |s| s.to_string()
));
named!(label_address(CompleteStr) -> Labeled, alt!(
  map!(address, |a| Labeled::Const(a)) |
  map!(label, |l| Labeled::Label(l))
));

named!(accumulator(CompleteStr) -> Register, alt!(
  value!(Register::A, tag_no_case!("A")) |
  value!(Register::B, tag_no_case!("B"))
));
named!(index(CompleteStr) -> Register, alt!(
  value!(Register::X, tag_no_case!("X")) |
  value!(Register::Y, tag_no_case!("Y"))
));
named!(register(CompleteStr) -> Register, alt!(accumulator | index));

named!(offset(CompleteStr) -> Address, alt!(
  map!(
    recognize!(pair!(one_of!("+-"), digit)),
    |s| Address::Offset(i8::from_str_radix(&s, 10).unwrap())
  ) |
  map!(label, |s| Address::Direct(Labeled::Label(s)))
));
named!(direct(CompleteStr) -> Address, map!(
  label_address,
  |addr| Address::Direct(addr)
));
named!(indexed(CompleteStr) -> Address, map!(
  sp!(separated_pair!(label_address, tag!("+"), index)),
  |(addr, index)| Address::Indexed(addr, index)
));
named!(indirect(CompleteStr) -> Address,
  map!(delimited!(tag!("("), label_address, tag!(")")), |addr| Address::Indirect(addr))
);
named!(indirect_indexed(CompleteStr) -> Address, map!(
  sp!(separated_pair!(delimited!(tag!("("), label_address, tag!(")")), tag!("+"), index)),
  |(addr, index)| Address::IndirectIndexed(addr, index)
));

named!(argument(CompleteStr) -> Argument, alt!(
  map!(byte, |b| Argument::Byte(b)) |
  map!(accumulator, |r| Argument::Register(r)) |
  map!(alt!(direct | indexed | indirect | indirect_indexed), |a| Argument::Address(a))
));

named!(nop(CompleteStr) -> Op, value!(Op::Nop, sp!(tag_no_case!("NOP"))));
named!(hlt(CompleteStr) -> Op, value!(Op::Hlt, sp!(tag_no_case!("HLT"))));
named!(brk(CompleteStr) -> Op, value!(Op::Brk, sp!(tag_no_case!("BRK"))));
named!(int(CompleteStr) -> Op, value!(Op::Int, sp!(tag_no_case!("INT"))));

named!(set(CompleteStr) -> Op, sp!(do_parse!(
  tag_no_case!("SET") >>
  flag: alt!(
    value!(Flag::C, tag_no_case!("C")) |
    value!(Flag::I, tag_no_case!("I"))
  ) >>
  comma >>
  value: alt!(
    value!(false, tag!("0")) |
    value!(true, tag!("1"))
  ) >>
  (Op::Set(flag, value))
)));

named!(call(CompleteStr) -> Op, sp!(do_parse!(
  tag_no_case!("CALL") >>
  target: alt!(direct | indirect | indirect_indexed) >>
  (Op::Call(target))
)));
named!(ret(CompleteStr) -> Op,  value!(Op::Ret, sp!(tag_no_case!("RET"))));
named!(reti(CompleteStr) -> Op, value!(Op::RetI, sp!(tag_no_case!("RETI"))));
named!(jmp(CompleteStr) -> Op,  sp!(alt!(
  do_parse!(
    tag_no_case!("JMP") >>
    addr: alt!(offset | direct) >>
    (Op::Jmp(None, addr))
  ) |
  do_parse!(
    tag_no_case!("JMP") >>
    cond: alt!(
      value!((Flag::Z, false), tag_no_case!("NZ")) |
      value!((Flag::Z,  true), tag_no_case!( "Z")) |
      value!((Flag::C, false), tag_no_case!("NC")) |
      value!((Flag::C,  true), tag_no_case!( "C")) |
      value!((Flag::V, false), tag_no_case!("NV")) |
      value!((Flag::V,  true), tag_no_case!( "V")) |
      value!((Flag::S, false), tag_no_case!( "P")) |
      value!((Flag::S,  true), tag_no_case!( "N"))
    ) >>
    comma >>
    addr: offset >>
    (Op::Jmp(Some(cond), addr))
  )
)));

macro_rules! acc_op (
  ($i:expr, $tag:expr, Op::$op:ident) => ({
    sp!($i, do_parse!(
      tag_no_case!($tag) >>
      dest: accumulator >>
      comma >>
      src: argument >>
      (Op::$op(dest, src))
    ))
  })
);
named!(add(CompleteStr) -> Op, sp!(alt!(
  acc_op!("ADD", Op::Add) |
  do_parse!(tag_no_case!("ADD") >> dest: index >> comma >> src: byte >> (Op::Add(dest, Argument::Byte(src))))
)));
named!(sub(CompleteStr) -> Op, sp!(alt!(
  acc_op!("SUB", Op::Sub) |
  do_parse!(tag_no_case!("SUB") >> dest: index >> comma >> src: byte >> (Op::Sub(dest, Argument::Byte(src))))
)));
named!(addc(CompleteStr) -> Op, acc_op!("ADDC", Op::AddC));
named!(subc(CompleteStr) -> Op, acc_op!("SUBC", Op::SubC));
named!(and(CompleteStr) -> Op, acc_op!("AND", Op::And));
named!(or(CompleteStr) -> Op, acc_op!("OR", Op::Or));
named!(xor(CompleteStr) -> Op, acc_op!("XOR", Op::Xor));
named!(cmp(CompleteStr) -> Op, sp!(alt!(
  acc_op!("CMP", Op::Cmp) |
  do_parse!(
    tag_no_case!("CMP") >>
    dest: index >>
    comma >>
    src: alt!(
      map!(byte, |b| Argument::Byte(b)) |
      map!(alt!(accumulator | index), |r| Argument::Register(r)) |
      map!(
        alt!(direct | indexed | indirect | indirect_indexed),
        |a| Argument::Address(a)
      )
    ) >>
    (Op::Cmp(dest, src))
  )
)));

named!(inc(CompleteStr) -> Op, map!(sp!(preceded!(tag_no_case!("INC"), register)), |r| Op::Inc(r)));
named!(dec(CompleteStr) -> Op, map!(sp!(preceded!(tag_no_case!("DEC"), register)), |r| Op::Dec(r)));
named!(not(CompleteStr) -> Op, map!(sp!(preceded!(tag_no_case!("NOT"), accumulator)), |r| Op::Not(r)));
named!(neg(CompleteStr) -> Op, map!(sp!(preceded!(tag_no_case!("NEG"), accumulator)), |r| Op::Neg(r)));
named!(rr(CompleteStr) -> Op, map!(sp!(preceded!(tag_no_case!("RR"), accumulator)), |r| Op::Rr(r)));
named!(rrc(CompleteStr) -> Op, map!(sp!(preceded!(tag_no_case!("RRC"), accumulator)), |r| Op::RrC(r)));
named!(rl(CompleteStr) -> Op, map!(sp!(preceded!(tag_no_case!("RL"), accumulator)), |r| Op::Rl(r)));
named!(rlc(CompleteStr) -> Op, map!(sp!(preceded!(tag_no_case!("RLC"), accumulator)), |r| Op::RlC(r)));
named!(push(CompleteStr) -> Op, map!(sp!(preceded!(tag_no_case!("PUSH"), register)), |r| Op::Push(r)));
named!(pop(CompleteStr) -> Op, map!(sp!(preceded!(tag_no_case!("POP"), register)), |r| Op::Pop(r)));

named!(ld(CompleteStr) -> Op, sp!(alt!(
  do_parse!(
    tag_no_case!("LD") >>
    dest: register >>
    comma >>
    src: argument >>
    (Op::Ld(Argument::Register(dest), src))
  ) |
  do_parse!(
    tag_no_case!("LD") >>
    dest: alt!(direct | indexed | indirect | indirect_indexed) >>
    comma >>
    src: register >>
    (Op::Ld(Argument::Address(dest), Argument::Register(src)))
  )
)));

named!(section(CompleteStr) -> Token, map!(
  sp!(preceded!(pair!(tag!(".section"), one_of!(":")), address)),
  |a| Token::Section(a)
));

named!(label_define(CompleteStr) -> Token, map!(
  sp!(terminated!(label, one_of!(":"))),
  |s| Token::Label(s)
));

named!(instruction(CompleteStr) -> Token, map!(
  alt!(
    nop | hlt | brk | int | set |
    call | ret | reti | jmp |
    add | addc | sub | subc | and | or | xor | cmp |
    neg | not | inc | dec | rr | rrc | rl | rlc |
    push | pop | ld
  ),
  |op| Token::Op(op)
));

named!(parser(CompleteStr) -> Vec<Token>, map!(
  separated_list!(
    alt!(tag!("\n") | tag!("\r\n")),
    alt!(
      map!(section, |s| vec![Some(s)]) |
      map!(pair!(opt!(label_define), opt!(instruction)), |(a, b)| vec![a, b])
    )
  ),
  |result: Vec<Vec<Option<Token>>>| -> Vec<Token> {
    result.into_iter().flatten().filter_map(|v| v).collect()
  }
));

pub fn parse(input: &str) -> Vec<Token> {
  let (remaining, parsed) = parser(CompleteStr(input)).unwrap();
  if remaining.len() > 0 {
    println!("Remaining: {:?}", remaining);
  }
  parsed
}

pub fn assemble(input: &str) -> Vec<u8> {
  let mut tokens = parse(input);

  let mut sections: Vec<(u16, Vec<Token>)> = Vec::new();
  let mut labels: HashMap<String, u16> = HashMap::new();

  match tokens[0] {
    Token::Section(_) => (),
    _ => tokens.insert(0, Token::Section(0x0000)),
  }

  let mut iter = tokens.into_iter().peekable();
  while let Some(token) = iter.next() {
    println!("AA={:?}", token);
    let (address, mut section) = match token {
      Token::Section(address) => (address, Vec::new()),
      _ => panic!("Beginning of section with no section token."),
    };

    let mut l_addr = Some(address);
    while let Some(token) = iter.peek() {
      match token {
        Token::Section(_) => break,
        Token::Op(_) => {
          if let Some(Token::Op(mut op)) = iter.next() {
            match l_addr {
              Some(l_addr) => op.resolve(l_addr, &labels),
              None => (),
            }
            l_addr = match (l_addr, op.length()) {
              (Some(a), Some(l)) => Some(a + l),
              _ => None,
            };
            section.push(Token::Op(op));
          }
        },
        Token::Label(label) => {
          match l_addr {
            Some(addr) => { labels.insert(label.clone(), addr); },
            None => (),
          }
          iter.next();
        },
      }
    }

    sections.push((address, section));
  }

  println!("Labels: {:?}\n", labels);

  let mut out: Vec<u8> = Vec::new();

  let first_section = sections[0].0;
  for (start, section) in sections.iter_mut() {
    for i in 0..(*start - first_section - (out.len() as u16)) {
      out.push(0x00);
    }
    for token in section.iter_mut() {
      match token {
        Token::Section(_) => panic!("How are there section tokens here."),
        Token::Label(op) => panic!("If there are still label tokens at this point then idk what to do lmao."),
        Token::Op(op) => {
          op.resolve(first_section + (out.len() as u16), &labels);
          op.assemble(&mut out);
        },
      }
    }
  }

  println!("Sections: {:?}\n", sections);

  out
}
