
use nom::types::CompleteStr;
use nom::{
  line_ending,
  not_line_ending,
};

use crate::assembler::error::Error;
use crate::assembler::tokens::*;


fn is_space(chr: char) -> bool {
  chr == ' ' || chr == '\t'
}
named!(sp0(CompleteStr) -> CompleteStr, take_while!(is_space));
named!(sp1(CompleteStr) -> CompleteStr, take_while1!(is_space));

fn is_hex_digit(chr: char) -> bool { chr.is_digit(16) }
fn is_bin_digit(chr: char) -> bool { chr.is_digit(2) }
fn is_digit(chr: char) -> bool { chr.is_digit(10) }
named!(number(CompleteStr) -> u16, alt!(
  preceded!(tag!("0x"), take_while1!(is_hex_digit)) => { |s: CompleteStr| u16::from_str_radix(&s, 16).unwrap() } |
  preceded!(tag!("0b"), take_while1!(is_bin_digit)) => { |s: CompleteStr| u16::from_str_radix(&s, 2).unwrap() } |
  recognize!(pair!(opt!(one_of!("+-")), take_while1!(is_digit))) => { |s: CompleteStr| i32::from_str_radix(&s, 10).unwrap() as u16 }
));

fn is_name_char(chr: char) -> bool {
  chr.is_alphanumeric() || chr == '_'
}
named!(name(CompleteStr) -> String, map!(
  recognize!(pair!(one_of!("."), take_while1!(is_name_char))),
  |s| s.to_string()
));

named!(operand(CompleteStr) -> Operand, alt!(
  one_of!("*") => { |_| Operand::Star } |
  number => { |n| Operand::Number(n) } |
  name => { |s| Operand::Name(s) }
));
named!(expression(CompleteStr) -> Expression, alt!(
  do_parse!(
    lhs: operand >>
    sp0 >>
    op: one_of!("+-") >>
    sp0 >>
    rhs: operand >>
    (match op {
      '+' => Expression::Add(lhs, rhs),
      '-' => Expression::Sub(lhs, rhs),
      _ => panic!("Nom failed at parsing expression."),
    })
  ) |
  do_parse!(
    op: one_of!("<>") >>
    sp0 >>
    o: operand >>
    (match op {
      '>' => Expression::High(o),
      '<' => Expression::Low(o),
      _ => panic!("Nom failed at parsing expression."),
    })
  ) |
  operand => { |e| Expression::Unary(e) }
));

named!(accumulator(CompleteStr) -> Register, alt!(
  tag_no_case!("A") => { |_| Register::A } |
  tag_no_case!("B") => { |_| Register::B }
));
named!(index(CompleteStr) -> Register, alt!(
  tag_no_case!("X") => { |_| Register::X } |
  tag_no_case!("Y") => { |_| Register::Y }
));
named!(register(CompleteStr) -> Register, alt!(accumulator | index));

named!(direct(CompleteStr) -> Address, do_parse!(
  tag!("@") >> sp0 >> e: expression >>
  (Address::Direct(e))
));
named!(indexed(CompleteStr) -> Address, do_parse!(
  tag!("@") >> sp0 >> e: expression >> sp0 >> tag!("+") >> sp0 >> i: index >>
  (Address::Indexed(e, i))
));
named!(indirect(CompleteStr) -> Address, do_parse!(
  tag!("(") >> sp0 >> tag!("@") >> sp0 >> e: expression >> sp0 >> tag!(")") >>
  (Address::Indirect(e))
));
named!(indirect_indexed(CompleteStr) -> Address, do_parse!(
  tag!("(") >> sp0 >> tag!("@") >> sp0 >> e: expression >> sp0 >> tag!(")") >>
  sp0 >> tag!("+") >> sp0 >> i: index >>
  (Address::IndirectIndexed(e, i))
));
named!(address(CompleteStr) -> Address,
  alt!(indirect_indexed | indirect | indexed | direct)
);

named!(argument(CompleteStr) -> Argument, alt!(
  address => { |a| Argument::Address(a) } |
  expression => { |b| Argument::Byte(b) } |
  register => { |r| Argument::Register(r) }
));

named!(nop(CompleteStr) -> Op, value!(Op::Nop, tag_no_case!("NOP")));
named!(hlt(CompleteStr) -> Op, value!(Op::Hlt, tag_no_case!("HLT")));
named!(brk(CompleteStr) -> Op, value!(Op::Brk, tag_no_case!("BRK")));
named!(int(CompleteStr) -> Op, value!(Op::Int, tag_no_case!("INT")));

named!(set(CompleteStr) -> Op, do_parse!(
  tag_no_case!("SET") >> sp1 >>
  flag: alt!(
    tag_no_case!("C") => { |_| Flag::C } |
    tag_no_case!("I") => { |_| Flag::I }
  ) >>
  sp0 >> one_of!(",") >> sp0 >>
  value: alt!(
    tag!("0") => { |_| false } |
    tag!("1") => { |_| true }
  ) >>
  (Op::Set(flag, value))
));

named!(call(CompleteStr) -> Op, do_parse!(
  tag_no_case!("CALL") >> sp1 >> a: address >>
  (Op::Call(a))
));
named!(ret(CompleteStr) -> Op,  value!(Op::Ret, tag_no_case!("RET")));
named!(reti(CompleteStr) -> Op, value!(Op::RetI, tag_no_case!("RETI")));
named!(jmp(CompleteStr) -> Op,  alt!(
  do_parse!(
    tag_no_case!("JMP") >> sp1 >>
    addr: direct >>
    (Op::Jmp(None, addr))
  ) |
  do_parse!(
    tag_no_case!("JMP") >> sp1 >>
    cond: alt!(
      tag_no_case!("NZ") => { |_| (Flag::Z, false) } |
      tag_no_case!( "Z") => { |_| (Flag::Z,  true) } |
      tag_no_case!("NC") => { |_| (Flag::C, false) } |
      tag_no_case!( "C") => { |_| (Flag::C,  true) } |
      tag_no_case!("NV") => { |_| (Flag::V, false) } |
      tag_no_case!( "V") => { |_| (Flag::V,  true) } |
      tag_no_case!( "P") => { |_| (Flag::S, false) } |
      tag_no_case!( "N") => { |_| (Flag::S,  true) }
    ) >>
    sp0 >> one_of!(",") >> sp0 >>
    addr: direct >>
    (Op::Jmp(Some(cond), addr))
  )
));

macro_rules! acc_op (
  ($i:expr, $tag:expr, Op::$op:ident) => ({
    do_parse!(
      $i,
      tag_no_case!($tag) >> sp1 >>
      dest: accumulator >>
      sp0 >> one_of!(",") >> sp0 >>
      src: argument >>
      (Op::$op(dest, src))
    )
  })
);
named!(add(CompleteStr) -> Op, alt!(
  acc_op!("ADD", Op::Add) |
  do_parse!(
    tag_no_case!("ADD") >> sp1 >> dest: index >> sp0 >> one_of!(",") >> sp0 >> src: expression >>
    (Op::Add(dest, Argument::Byte(src)))
  )
));
named!(sub(CompleteStr) -> Op, alt!(
  acc_op!("SUB", Op::Sub) |
  do_parse!(
    tag_no_case!("SUB") >> sp1 >> dest: index >> sp0 >> one_of!(",") >> sp0 >> src: expression >>
    (Op::Sub(dest, Argument::Byte(src)))
  )
));
named!(addc(CompleteStr) -> Op, acc_op!("ADDC", Op::AddC));
named!(subc(CompleteStr) -> Op, acc_op!("SUBC", Op::SubC));
named!(and(CompleteStr) -> Op, acc_op!("AND", Op::And));
named!(or(CompleteStr) -> Op, acc_op!("OR", Op::Or));
named!(xor(CompleteStr) -> Op, acc_op!("XOR", Op::Xor));
named!(cmp(CompleteStr) -> Op, alt!(
  acc_op!("CMP", Op::Cmp) |
  do_parse!(
    tag_no_case!("CMP") >> sp1 >> dest: index >> sp0 >> one_of!(",") >> sp0 >> src: argument >>
    (Op::Cmp(dest, src))
  )
));

named!( inc(CompleteStr) -> Op, do_parse!(tag_no_case!( "INC") >> sp1 >> r: register >> (Op::Inc(r))));
named!( dec(CompleteStr) -> Op, do_parse!(tag_no_case!( "DEC") >> sp1 >> r: register >> (Op::Dec(r))));
named!( not(CompleteStr) -> Op, do_parse!(tag_no_case!( "NOT") >> sp1 >> r: accumulator >> (Op::Not(r))));
named!( neg(CompleteStr) -> Op, do_parse!(tag_no_case!( "NEG") >> sp1 >> r: accumulator >> (Op::Neg(r))));
named!(  rr(CompleteStr) -> Op, do_parse!(tag_no_case!(  "RR") >> sp1 >> r: accumulator >> (Op::Rr(r))));
named!( rrc(CompleteStr) -> Op, do_parse!(tag_no_case!( "RRC") >> sp1 >> r: accumulator >> (Op::RrC(r))));
named!(  rl(CompleteStr) -> Op, do_parse!(tag_no_case!(  "RL") >> sp1 >> r: accumulator >> (Op::Rl(r))));
named!( rlc(CompleteStr) -> Op, do_parse!(tag_no_case!( "RLC") >> sp1 >> r: accumulator >> (Op::RlC(r))));
named!(push(CompleteStr) -> Op, do_parse!(tag_no_case!("PUSH") >> sp1 >> r: register >> (Op::Push(r))));
named!( pop(CompleteStr) -> Op, do_parse!(tag_no_case!( "POP") >> sp1 >> r: register >> (Op::Pop(r))));

named!(ld(CompleteStr) -> Op, alt!(
  do_parse!(
    tag_no_case!("LD") >> sp1 >>
    dest: register >>
    sp0 >> one_of!(",") >> sp0 >>
    src: argument >>
    (Op::Ld(Argument::Register(dest), src))
  ) |
  do_parse!(
    tag_no_case!("LD") >> sp1 >>
    dest: address >>
    sp0 >> one_of!(",") >> sp0 >>
    src: register >>
    (Op::Ld(Argument::Address(dest), Argument::Register(src)))
  )
));


named!(string(CompleteStr) -> String, alt!(
  do_parse!(one_of!("'") >> s: is_not!("\r\n'") >> one_of!("'") >> (s.to_string())) |
  do_parse!(one_of!("\"") >> s: is_not!("\r\n\"") >> one_of!("\"") >> (s.to_string()))
));

named!(comment(CompleteStr) -> String, map!(
  preceded!(tag!("//"), not_line_ending),
  |s| s.to_string()
));

named!(section(CompleteStr) -> Directive, do_parse!(
  tag_no_case!("#section") >> sp1 >> w: number >>
  (Directive::Section(w as u16))
));
named!(word_directive(CompleteStr) -> Directive, do_parse!(
  tag_no_case!("#word") >> sp1 >>
  w: separated_list!(one_of!(","), delimited!(sp0, expression, sp0)) >>
  (Directive::Word(w))
));
named!(byte_directive(CompleteStr) -> Directive, do_parse!(
  tag_no_case!("#byte") >> sp1 >>
  b: separated_list!(one_of!(","), delimited!(sp0, alt!(
    expression => { |e| -> Vec<Expression> { vec![e] } } |
    string => { |s: String| -> Vec<Expression> {
      s.into_bytes().into_iter()
        .map(|c| Expression::Unary(Operand::Number(c as u16)))
        .collect()
    } }
  ), sp0)) >>
  (Directive::Byte(b.into_iter().flatten().collect()))
));
named!(define(CompleteStr) -> Directive, do_parse!(
  tag_no_case!("#define") >> sp1 >>
  n: name >>
  sp0 >> one_of!("=") >> sp0 >>
  expr: expression >>
  (Directive::Define(n, expr))
));
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
    directive   => { |d| vec![d] } |
    instruction => { |i| vec![i] } |
    label       => { |l| vec![l] } |
    do_parse!(l: label >> sp1 >> d: directive >> ((l, d))) => { |(l, d)| vec![l, d] } |
    do_parse!(l: label >> sp1 >> i: instruction >> ((l, i))) => { |(l, i)| vec![l, i] } |
    comment     => { |_| vec![] }
  ),
  opt!(pair!(sp0, comment))
));

fn is_line_sep(chr: char) -> bool {
  chr == '\r' || chr == '\n'
}
fn is_line_sep_sp(chr: char) -> bool {
  chr == ' ' || chr == '\t' || chr == '\r' || chr == '\n'
}
named!(line_sep(CompleteStr) -> CompleteStr, 
  recognize!(pair!(take_while1!(is_line_sep), take_while!(is_line_sep_sp)))
);

named!(parser(CompleteStr) -> Vec<Token>, delimited!(
  opt!(line_sep),
  map!(
    separated_list!(line_sep, delimited!(sp0, line, sp0)),
    |line: Vec<Vec<Token>>| -> Vec<Token> {
      line.into_iter().flatten().collect()
    }
  ),
  opt!(line_sep)
));

pub fn parse(input: &str) -> Result<Vec<Token>, Error> {
  let (remaining, parsed) = parser(CompleteStr(input)).unwrap(); // TODO ERROR
  if remaining.len() > 0 {
    Err(Error::IncompleteParse(remaining.to_string()))
  } else {
    Ok(parsed)
  }
}
