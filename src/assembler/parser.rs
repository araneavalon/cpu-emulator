
use nom::types::CompleteStr;
use nom::{
  digit,
  alphanumeric1,
  line_ending,
  not_line_ending,
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


named!(hex_digit(CompleteStr) -> char, one_of!("0123456789ABCDEFabcdef"));
named!(byte(CompleteStr) -> u8, alt!(
  map!(preceded!(tag!("0x"), recognize!(many_m_n!(2, 2, hex_digit))), |s| u8::from_str_radix(&s, 16).unwrap()) |
  map!(preceded!(tag!("0b"), recognize!(many_m_n!(8, 8, one_of!("01")))), |s| u8::from_str_radix(&s, 2).unwrap()) |
  map!(recognize!(pair!(opt!(char!('+')), digit)), |s| u8::from_str_radix(&s, 10).unwrap()) |
  map!(recognize!(pair!(char!('-'), digit)), |s| i8::from_str_radix(&s, 10).unwrap() as u8)
));
named!(word(CompleteStr) -> u16,
  map!(preceded!(tag!("0x"), recognize!(many_m_n!(4, 4, hex_digit))), |s| u16::from_str_radix(&s, 16).unwrap())
);

named!(name(CompleteStr) -> String, map!(
  recognize!(pair!(char!('.'), alphanumeric1)),
  |s| s.to_string()
));

named!(unary_expr(CompleteStr) -> UnaryExpr, alt!(
  map!(char!('*'), |_| UnaryExpr::Star) |
  map!(word, |w| UnaryExpr::Value(Value::Word(w))) |
  map!(byte, |b| UnaryExpr::Value(Value::Byte(b))) |
  map!(name, |n| UnaryExpr::Name(n))
));
named!(expression(CompleteStr) -> Expression, alt!(
  sp!(do_parse!(
    lhs: unary_expr >>
    op: one_of!("+-") >>
    rhs: unary_expr >>
    (match op {
      '+' => Expression::Add(lhs, rhs),
      '-' => Expression::Sub(lhs, rhs),
      _ => panic!("Nom failed at parsing expxression."),
    })
  )) |
  map!(sp!(preceded!(char!('>'), unary_expr)), |e| Expression::High(e)) |
  map!(sp!(preceded!(char!('<'), unary_expr)), |e| Expression::Low(e)) |
  map!(unary_expr, |e| Expression::Unary(e))
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
  expression,
  |expr| Address::Direct(expr)
));
named!(indexed(CompleteStr) -> Address, map!(
  sp!(separated_pair!(expression, tag!("+"), index)),
  |(expr, index)| Address::Indexed(expr, index)
));
named!(indirect(CompleteStr) -> Address, map!(
  delimited!(tag!("("), expression, tag!(")")),
  |expr| Address::Indirect(expr)
));
named!(indirect_indexed(CompleteStr) -> Address, map!(
  sp!(separated_pair!(delimited!(tag!("("), expression, tag!(")")), tag!("+"), index)),
  |(expr, index)| Address::IndirectIndexed(expr, index)
));

named!(argument(CompleteStr) -> Argument, alt!(
  map!(expression, |b| Argument::Byte(b)) |
  map!(register, |r| Argument::Register(r)) |
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
  char!(',') >>
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
named!(jmp(CompleteStr) -> Op,  sp!(do_parse!(
  tag_no_case!("JMP") >>
  cond: opt!(sp!(terminated!(alt!(
    value!((Flag::Z, false), tag_no_case!("NZ")) |
    value!((Flag::Z,  true), tag_no_case!( "Z")) |
    value!((Flag::C, false), tag_no_case!("NC")) |
    value!((Flag::C,  true), tag_no_case!( "C")) |
    value!((Flag::V, false), tag_no_case!("NV")) |
    value!((Flag::V,  true), tag_no_case!( "V")) |
    value!((Flag::S, false), tag_no_case!( "P")) |
    value!((Flag::S,  true), tag_no_case!( "N"))
  ), char!(',')))) >>
  addr: direct >>
  (Op::Jmp(cond, addr))
)));

macro_rules! acc_op (
  ($i:expr, $tag:expr, Op::$op:ident) => ({
    sp!($i, do_parse!(
      tag_no_case!($tag) >>
      dest: accumulator >>
      char!(',') >>
      src: argument >>
      (Op::$op(dest, src))
    ))
  })
);
named!(add(CompleteStr) -> Op, sp!(alt!(
  acc_op!("ADD", Op::Add) |
  do_parse!(tag_no_case!("ADD") >> dest: index >> char!(',') >> src: expression >> (Op::Add(dest, Argument::Byte(src))))
)));
named!(sub(CompleteStr) -> Op, sp!(alt!(
  acc_op!("SUB", Op::Sub) |
  do_parse!(tag_no_case!("SUB") >> dest: index >> char!(',') >> src: expression >> (Op::Sub(dest, Argument::Byte(src))))
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
    char!(',') >>
    src: argument >>
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
    char!(',') >>
    src: argument >>
    (Op::Ld(Argument::Register(dest), src))
  ) |
  do_parse!(
    tag_no_case!("LD") >>
    dest: alt!(direct | indexed | indirect | indirect_indexed) >>
    char!(',') >>
    src: register >>
    (Op::Ld(Argument::Address(dest), Argument::Register(src)))
  )
)));


named!(string(CompleteStr) -> String, map!(
  delimited!(char!('\''), is_not!("\'"), char!('\'')),
  |s| s.to_string()
));

named!(comment(CompleteStr) -> String, map!(
  preceded!(tag!(";"), not_line_ending),
  |s| s.to_string()
));

named!(section(CompleteStr) -> Directive, map!(
  sp!(preceded!(tag_no_case!("#section"), word)),
  |a| Directive::Section(a)
));
named!(word_directive(CompleteStr) -> Directive, map!(
  sp!(preceded!(tag_no_case!("#word"), separated_list!(char!(','), expression))),
  |w| Directive::Word(w)
));
named!(byte_directive(CompleteStr) -> Directive, map!(
  sp!(preceded!(
    tag_no_case!("#byte"),
    sp!(separated_list!(char!(','), alt!(
      map!(expression, |b| vec![b]) |
      map!(string, |s| -> Vec<Expression> {
        s.into_bytes().into_iter()
          .map(|c| Expression::Unary(UnaryExpr::Value(Value::Byte(c))))
          .collect()
      })
    )))
  )),
  |result: Vec<Vec<Expression>>| -> Directive {
    Directive::Byte(result.into_iter().flatten().collect())
  }
));
named!(define(CompleteStr) -> Directive, sp!(do_parse!(
  tag_no_case!("#define") >>
  n: name >>
  char!('=') >>
  expr: expression >>
  (Directive::Define(n, expr))
)));
named!(directive(CompleteStr) -> Token, map!(
  alt!(section | word_directive | byte_directive | define),
  |d| Token::Directive(d)
));

named!(label(CompleteStr) -> Token, map!(name, |s| Token::Label(s)));

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

named!(line(CompleteStr) -> Vec<Token>, terminated!(
  alt!(
    map!(directive, |d| vec![d]) |
    map!(sp!(pair!(label, directive)), |(l, d)| vec![l, d]) |
    map!(instruction, |i| vec![i]) |
    map!(sp!(pair!(label, instruction)), |(l, i)| vec![l, i]) |
    map!(label, |l| vec![l])
  ),
  opt!(comment)
));

named!(line_sep(CompleteStr) -> CompleteStr,
  recognize!(sp!(many1!(line_ending)))
);

named!(parser(CompleteStr) -> Vec<Token>, sp!(delimited!(
  opt!(line_sep),
  map!(
    separated_list!(line_sep, line),
    |line: Vec<Vec<Token>>| -> Vec<Token> {
      line.into_iter().flatten().collect()
    }
  ),
  opt!(line_sep)
)));

pub fn parse(input: &str) -> Result<Vec<Token>, Error> {
  let (remaining, parsed) = parser(CompleteStr(input)).unwrap();
  if remaining.len() > 0 {
    Err(Error::IncompleteParse(remaining.to_string()))
  } else {
    Ok(parsed)
  }
}
