
use nom::types::CompleteStr;
use nom::{
  digit,
  hex_digit,
  alpha1,
  alphanumeric0,
};

#[derive(Debug)]
pub struct Label(String);
#[derive(Debug)]
pub enum Flag { Z, C, V, S, I }
#[derive(Debug)]
pub enum Register { A, B, X, Y }
#[derive(Debug)]
pub enum Address {
  Offset(i8),
  Address(u16),
  Indexed(u16, Register),
  Indirect(u16),
  IndirectIndexed(u16, Register),
}
#[derive(Debug)]
pub enum Argument {
  Byte(u8),
  Register(Register),
  Address(Address),
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
  Rrc(Register),
  Rl(Register),
  Rlc(Register),
  Push(Register),
  Pop(Register),
  Ld(Argument, Argument),
}


named!(separator(CompleteStr) -> char,
  one_of!(", ")
);

named!(byte(CompleteStr) -> u8, alt!(
  map!(preceded!(tag!("0x"), hex_digit), |s| { println!("{}", s); u8::from_str_radix(&s, 16).unwrap() }) |
  map!(recognize!(pair!(opt!(one_of!("+-")), digit)), |s| i16::from_str_radix(&s, 10).unwrap() as u8)
));
named!(address(CompleteStr) -> u16,
  map!(preceded!(tag!("0x"), hex_digit), |s| u16::from_str_radix(&s, 16).unwrap())
);

named!(accumulator(CompleteStr) -> Register, alt!(
  value!(Register::A, tag_no_case!("A")) |
  value!(Register::B, tag_no_case!("B"))
));
named!(index(CompleteStr) -> Register, alt!(
  value!(Register::X, tag_no_case!("X")) |
  value!(Register::Y, tag_no_case!("Y"))
));
named!(register(CompleteStr) -> Register, alt!(accumulator | index));

named!(offset(CompleteStr) -> Address, map!(
  recognize!(pair!(one_of!("+-"), digit)),
  |s| Address::Offset(i8::from_str_radix(&s, 10).unwrap())
));
named!(direct(CompleteStr) -> Address,
  map!(address, |addr| Address::Address(addr))
);
named!(indexed(CompleteStr) -> Address, map!(
  ws!(separated_pair!(address, tag!("+"), index)),
  |(addr, index)| Address::Indexed(addr, index)
));
named!(indirect(CompleteStr) -> Address,
  map!(delimited!(tag!("("), address, tag!(")")), |addr| Address::Indirect(addr))
);
named!(indirect_indexed(CompleteStr) -> Address, map!(
  ws!(separated_pair!(delimited!(tag!("("), address, tag!(")")), tag!("+"), index)),
  |(addr, index)| Address::IndirectIndexed(addr, index)
));

named!(argument(CompleteStr) -> Argument, alt!(
  map!(byte, |b| Argument::Byte(b)) |
  map!(accumulator, |r| Argument::Register(r)) |
  map!(alt!(direct | indexed | indirect | indirect_indexed), |a| Argument::Address(a))
));

named!(nop(CompleteStr) -> Op, value!(Op::Nop, tag_no_case!("NOP")));
named!(hlt(CompleteStr) -> Op, value!(Op::Hlt, tag_no_case!("HLT")));
named!(brk(CompleteStr) -> Op, value!(Op::Brk, tag_no_case!("BRK")));
named!(int(CompleteStr) -> Op, value!(Op::Int, tag_no_case!("INT")));

named!(set(CompleteStr) -> Op, ws!(do_parse!(
  tag_no_case!("SET") >>
  flag: alt!(
    value!(Flag::C, tag_no_case!("C")) |
    value!(Flag::I, tag_no_case!("I"))
  ) >>
  separator >>
  value: alt!(
    value!(false, tag!("0")) |
    value!(true, tag!("1"))
  ) >>
  (Op::Set(flag, value))
)));

named!(call(CompleteStr) -> Op, ws!(do_parse!(
  tag_no_case!("CALL") >>
  target: alt!(direct | indirect | indirect_indexed) >>
  (Op::Call(target))
)));
named!(ret(CompleteStr) -> Op,  value!(Op::Ret, tag_no_case!("RET")));
named!(reti(CompleteStr) -> Op, value!(Op::RetI, tag_no_case!("RETI")));
named!(jmp(CompleteStr) -> Op,  alt!(
  ws!(do_parse!(
    tag_no_case!("JMP") >>
    addr: alt!(offset | direct) >>
    (Op::Jmp(None, addr))
  )) |
  ws!(do_parse!(
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
    separator >>
    addr: offset >>
    (Op::Jmp(Some(cond), addr))
  ))
));

named!(add(CompleteStr) -> Op, alt!(
  ws!(do_parse!(tag_no_case!("ADD") >> dest: accumulator >> separator >> src: argument >> (Op::Add(dest, src)))) |
  ws!(do_parse!(tag_no_case!("ADD") >> dest: index >> separator >> src: byte >> (Op::Add(dest, Argument::Byte(src)))))
));
named!(addc(CompleteStr) -> Op, ws!(do_parse!(tag_no_case!("ADDC") >> dest: accumulator >> separator >> src: argument >> (Op::AddC(dest, src)))));
named!(sub(CompleteStr) -> Op,  ws!(do_parse!(tag_no_case!("SUB")  >> dest: accumulator >> separator >> src: argument >> (Op::Sub(dest, src)))));
named!(subc(CompleteStr) -> Op, ws!(do_parse!(tag_no_case!("SUBC") >> dest: accumulator >> separator >> src: argument >> (Op::SubC(dest, src)))));
named!(and(CompleteStr) -> Op,  ws!(do_parse!(tag_no_case!("AND")  >> dest: accumulator >> separator >> src: argument >> (Op::And(dest, src)))));
named!(or(CompleteStr) -> Op,   ws!(do_parse!(tag_no_case!("OR")   >> dest: accumulator >> separator >> src: argument >> (Op::Or(dest, src)))));
named!(xor(CompleteStr) -> Op,  ws!(do_parse!(tag_no_case!("XOR")  >> dest: accumulator >> separator >> src: argument >> (Op::Xor(dest, src)))));
named!(cmp(CompleteStr) -> Op, alt!(
  ws!(do_parse!(
    tag_no_case!("CMP") >>
    dest: accumulator >>
    separator >>
    src: argument >>
    (Op::Cmp(dest, src))
  )) |
  ws!(do_parse!(
    tag_no_case!("CMP") >>
    dest: index >>
    separator >>
    src: alt!(
      map!(byte, |b| Argument::Byte(b)) |
      map!(alt!(accumulator | index), |r| Argument::Register(r)) |
      map!(
        alt!(direct | indexed | indirect | indirect_indexed),
        |a| Argument::Address(a)
      )
    ) >>
    (Op::Cmp(dest, src))
  ))
));

named!(inc(CompleteStr) -> Op, map!(ws!(preceded!(tag_no_case!("INC"), register)), |r| Op::Inc(r)));
named!(dec(CompleteStr) -> Op, map!(ws!(preceded!(tag_no_case!("DEC"), register)), |r| Op::Dec(r)));
named!(not(CompleteStr) -> Op, map!(ws!(preceded!(tag_no_case!("NOT"), accumulator)), |r| Op::Not(r)));
named!(neg(CompleteStr) -> Op, map!(ws!(preceded!(tag_no_case!("NEG"), accumulator)), |r| Op::Neg(r)));
named!(rr(CompleteStr) -> Op, map!(ws!(preceded!(tag_no_case!("RR"), accumulator)), |r| Op::Rr(r)));
named!(rrc(CompleteStr) -> Op, map!(ws!(preceded!(tag_no_case!("RRC"), accumulator)), |r| Op::Rrc(r)));
named!(rl(CompleteStr) -> Op, map!(ws!(preceded!(tag_no_case!("RL"), accumulator)), |r| Op::Rl(r)));
named!(rlc(CompleteStr) -> Op, map!(ws!(preceded!(tag_no_case!("RLC"), accumulator)), |r| Op::Rlc(r)));

named!(push(CompleteStr) -> Op, map!(ws!(preceded!(tag_no_case!("PUSH"), register)), |r| Op::Push(r)));
named!(pop(CompleteStr) -> Op, map!(ws!(preceded!(tag_no_case!("POP"), register)), |r| Op::Pop(r)));
named!(ld(CompleteStr) -> Op, alt!(
  ws!(do_parse!(
    tag_no_case!("LD") >>
    dest: register >>
    separator >>
    src: argument >>
    (Op::Ld(Argument::Register(dest), src))
  )) |
  ws!(do_parse!(
    tag_no_case!("LD") >>
    dest: alt!(direct | indexed | indirect | indirect_indexed) >>
    separator >>
    src: register >>
    (Op::Ld(Argument::Address(dest), Argument::Register(src)))
  ))
));

named!(instruction(CompleteStr) -> Op, alt!(
  nop | hlt | brk | int | set |
  call | ret | reti | jmp |
  add | addc | sub | subc | and | or | xor | cmp |
  neg | not | inc | dec | rr | rrc | rl | rlc |
  push | pop | ld
));

// TODO make this actually require newlines and not just gobble them up with ws!
named!(parser(CompleteStr) -> Vec<Op>,
  ws!(many0!(instruction))
);


pub fn parse(input: &str) -> Vec<Op> {
  let (remaining, parsed) = parser(CompleteStr(input)).unwrap();
  if remaining.len() > 0 {
    println!("Remaining: {:?}", remaining);
  }
  parsed
}
