
use nom::types::CompleteStr;
use nom::{
  digit,
  hex_digit,
  alphanumeric1,
};

use crate::assembler::error::Error;
use crate::assembler::tokens::*;


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
named!(address_target(CompleteStr) -> AddressTarget, alt!(
  map!(address, |a| AddressTarget::Address(a)) |
  map!(label, |l| AddressTarget::Label(l))
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

named!(direct(CompleteStr) -> Address, map!(
  address_target,
  |target| Address::Direct(target)
));
named!(indexed(CompleteStr) -> Address, map!(
  sp!(separated_pair!(address_target, tag!("+"), index)),
  |(target, index)| Address::Indexed(target, index)
));
named!(indirect(CompleteStr) -> Address, map!(
  delimited!(tag!("("), address_target, tag!(")")),
  |target| Address::Indirect(target)
));
named!(indirect_indexed(CompleteStr) -> Address, map!(
  sp!(separated_pair!(delimited!(tag!("("), address_target, tag!(")")), tag!("+"), index)),
  |(target, index)| Address::IndirectIndexed(target, index)
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
    addr: direct >>
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
    addr: direct >>
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

named!(section(CompleteStr) -> u16,
  sp!(preceded!(pair!(tag!(".section"), one_of!(":")), address))
);

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

named!(line_sep(CompleteStr) -> Vec<CompleteStr>,
  many1!(alt!(tag!("\n") | tag!("\r\n")))
);

named!(parser(CompleteStr) -> Vec<(u16, Vec<Token>)>, many0!(
  delimited!(
    opt!(line_sep),
    separated_pair!(
      section,
      line_sep,
      map!(
        separated_list!(line_sep, alt!(
            map!(instruction, |i| vec![i]) |
            map!(pair!(label_define, instruction), |(l, i)| vec![l, i]) |
            map!(separated_pair!(label_define, line_sep, instruction), |(l, i)| vec![l, i])
        )),
        |result: Vec<Vec<Token>>| -> Vec<Token> {
          result.into_iter().flatten().collect()
        }
      )
    ),
    opt!(line_sep)
  )
));

pub fn parse(input: &str) -> Result<Vec<(u16, Vec<Token>)>, Error> {
  let (remaining, parsed) = parser(CompleteStr(input)).unwrap();
  if remaining.len() > 0 {
    println!("Remaining: {:?}", remaining);
  }
  Ok(parsed)
}
