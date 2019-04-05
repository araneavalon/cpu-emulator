
// Notes:

// #section 0x0000
// #define IDENT = 0x0000
// #word 0x0000
// #byte 0x00,"string",0x00,0x00
// #byte directives are always aligned in blocks.
// LABEL:
// +1 / -1 (Refers to closest label with that name forward or backward, can only be used with single digit numbers.)
// * is alias for PC in word directive, can +/-

// n 000 Always    "  "       / " !"
// n 001 Always    "  "       / " !"
// n 010 Zero      "Z.", "E." / "Z!", "E!"
// n 011 Sign      "N.", "P!" / "N!", "P."
// n 100 Carry     "C.", "< " / "C!", ">="
// n 101 C&!Z            "> " /       "<="
// n 110 oVerflow  "V.", "Lt" / "V!", "Ge"
// n 111 V&!Z            "Gt" /       "Le"

// Registers are R0..R7,S0,S1,PC,LR(,F) (Registers can be A,B,C,D,E,X,Y,Z Stacks can be SR,SD)
// LD r,VALUE
//     [0,255] -> LD r,ub
//     [-128,127] -> LD r,sb
//     (,-129]|[256,) -> LD r,word

// LD r,(s+0) // in general, a bad idea.

// PUT0=PUTS=PUT, PUT1=PUTD

// OPb -> byte mode
// OP=OPw -> word mode
// SUB A,B -> A:=A-B
// SBN A,B -> A:=B-A


use std::collections::HashMap;
use nom::{
  types::CompleteStr,
  line_ending,
  alphanumeric0,
  alpha1,
  is_digit,
  hex_digit1,
  digit1,
};


#[derive(Debug)]
enum Directive {
  Define,
  Literal,
  Section,
}

#[derive(Debug)]
enum Condition {
  Zero,
  Sign,
  Carry,
  Overflow,
  CarryAndNotZero,
  OverflowAndNotZero,
}

#[derive(Debug)]
enum Op {
  Ld,
  Add,
  Sub,
  Sbn,
  Cmp,
  Cpn,
  And,
  Or,
  Xor,
  Put,
  Pop,
  Jmp,
  Test,
  Set,
  Int,
  Brk,
  Hlt,
  Nop,
}

#[derive(Debug)]
enum Mode {
  Byte,
  Signed,
  Negate,
  Stack,
  Indirect,
  Indexed,
  StackOffset,
}

#[derive(Debug)]
enum Register {
  Zero,
  One,
  Two,
  Three,
  Four,
  Five,
  Six,
  Seven,
  StackZero,
  StackOne,
  ProgramCounter,
  LinkRegister,
  Flags,
}

#[derive(Debug)]
enum Symbol<'a> {
  Directive(Directive),
  Label(&'a str),
  Word(u16),
  Byte(u8),
  Condition(Condition),
  Op(Op),
  Mode(Mode),
  Register(Register),
}

named!(word(CompleteStr) -> Symbol, map!(
  alt!(
    map_res!(preceded!(tag!("0x"), hex_digit1), |v: CompleteStr| { u16::from_str_radix(&v, 16) }) |
    map_res!(recognize!(preceded!(opt!(one_of!("+")), digit1)), |v: CompleteStr| u16::from_str_radix(&v, 10)) |
    map_res!(
      recognize!(preceded!(one_of!("-"), digit1)),
      |v: CompleteStr| match i16::from_str_radix(&v, 10) {
        Err(e) => Err(e),
        Ok(v) => Ok(v as u16),
      }
    )
  ),
  |v: u16| -> Symbol { Symbol::Word(v) }
));

named!(byte(CompleteStr) -> Symbol, map!(
  alt!(
    map_res!(preceded!(tag!("0x"), hex_digit1), |v: CompleteStr| u8::from_str_radix(&v, 16)) |
    map_res!(recognize!(preceded!(opt!(one_of!("+")), digit1)), |v: CompleteStr| u8::from_str_radix(&v, 10)) |
    map_res!(
      recognize!(preceded!(one_of!("-"), digit1)),
      |v: CompleteStr| match i8::from_str_radix(&v, 10) {
        Err(e) => Err(e),
        Ok(v) => Ok(v as u8),
      }
    )
  ),
  |v| Symbol::Byte(v)
));

named!(nibble(CompleteStr) -> Symbol, map!(
  map_opt!(one_of!("01234567"), |v: char| v.to_digit(10)),
  |v| Symbol::Byte(v as u8)
));

named!(register(CompleteStr) -> Symbol, alt!(
  value!(Symbol::Register(Register::Zero),  alt!(tag_no_case!("R0") | tag_no_case!("A"))) |
  value!(Symbol::Register(Register::One),   alt!(tag_no_case!("R1") | tag_no_case!("B"))) |
  value!(Symbol::Register(Register::Two),   alt!(tag_no_case!("R2") | tag_no_case!("C"))) |
  value!(Symbol::Register(Register::Three), alt!(tag_no_case!("R3") | tag_no_case!("D"))) |
  value!(Symbol::Register(Register::Four),  alt!(tag_no_case!("R4") | tag_no_case!("E"))) |
  value!(Symbol::Register(Register::Five),  alt!(tag_no_case!("R5") | tag_no_case!("X"))) |
  value!(Symbol::Register(Register::Six),   alt!(tag_no_case!("R6") | tag_no_case!("Y"))) |
  value!(Symbol::Register(Register::Seven), alt!(tag_no_case!("R7") | tag_no_case!("Z")))
));

named!(flags_register(CompleteStr) -> Symbol,
  value!(Symbol::Register(Register::Flags), tag_no_case!("F"))
);

named!(stack_register(CompleteStr) -> Symbol, alt!(
  value!(Symbol::Register(Register::StackZero), alt!(tag_no_case!("S0") | tag_no_case!("SR"))) |
  value!(Symbol::Register(Register::StackOne),  alt!(tag_no_case!("S1") | tag_no_case!("SD")))
));

named!(program_register(CompleteStr) -> Symbol, alt!(
  value!(Symbol::Register(Register::ProgramCounter), tag_no_case!("PC")) |
  value!(Symbol::Register(Register::LinkRegister),   tag_no_case!("LR"))
));

named!(immediate(CompleteStr) -> Vec<Symbol>, alt!(
  map!(
    map_res!(preceded!(tag!("0x"), hex_digit1), |v: CompleteStr| u8::from_str_radix(&v, 16)),
    |v| vec![Symbol::Byte(v)]
  ) |
  map!(
    map_res!(recognize!(preceded!(opt!(one_of!("+")), digit1)), |v: CompleteStr| u8::from_str_radix(&v, 10)),
    |v| vec![Symbol::Byte(v)]
  ) |
  map_res!(
    recognize!(preceded!(one_of!("-"), digit1)),
    |v: CompleteStr| match i8::from_str_radix(&v, 10) {
      Err(e) => Err(e),
      Ok(v) => Ok(vec![Symbol::Mode(Mode::Signed), Symbol::Byte(v as u8)]),
    }
  )
));

named!(offset(CompleteStr) -> Vec<Symbol>, map!(
  alt!(
    map_res!(preceded!(tag!("0x"), hex_digit1), |v: CompleteStr| i8::from_str_radix(&v, 16)) |
    map_res!(recognize!(preceded!(opt!(one_of!("+-")), digit1)), |v: CompleteStr| i8::from_str_radix(&v, 10))
  ),
  |v| vec![Symbol::Mode(Mode::Signed), Symbol::Byte(v as u8)]
));

named!(indexed(CompleteStr) -> Vec<Symbol>, do_parse!(
  one_of!("(") >>
  b: register >>
  one_of!("+") >>
  i: register >>
  one_of!(")") >>
  (vec![Symbol::Mode(Mode::Indexed), b, i])
));

named!(stack_offset(CompleteStr) -> Vec<Symbol>, do_parse!(
  one_of!("(") >>
  s: stack_register >>
  one_of!("+") >>
  o: nibble >>
  one_of!(")") >>
  (vec![Symbol::Mode(Mode::StackOffset), s, o])
));

named!(indirect(CompleteStr) -> Vec<Symbol>, map!(
  delimited!(one_of!("("), register, one_of!(")")),
  |r| vec![Symbol::Mode(Mode::Indirect), r]
));

named!(variable(CompleteStr) -> Vec<Symbol>, map!(
  delimited!(one_of!("("), word, one_of!(")")),
  |w| vec![Symbol::Mode(Mode::Indirect), w]
));

named!(stack(CompleteStr) -> Vec<Symbol>, delimited!(
  one_of!("["),
  separated_list!(one_of!(","), alt!(register | flags_register | program_register)),
  one_of!("]")
));

named!(condition(CompleteStr) -> Vec<Symbol>, alt!(
  value!(
    vec![Symbol::Mode(Mode::Negate)],
    tag_no_case!("!")
  ) |
  value!(
    vec![Symbol::Condition(Condition::Zero)],
    alt!(tag_no_case!("Z.") | tag_no_case!("E."))
  ) |
  value!(
    vec![Symbol::Mode(Mode::Negate), Symbol::Condition(Condition::Zero)],
    alt!(tag_no_case!("Z!") | tag_no_case!("E!"))
  ) |
  value!(
    vec![Symbol::Condition(Condition::Sign)],
    alt!(tag_no_case!("N.") | tag_no_case!("P!"))
  ) |
  value!(
    vec![Symbol::Mode(Mode::Negate), Symbol::Condition(Condition::Sign)],
    alt!(tag_no_case!("N!") | tag_no_case!("P."))
  ) |
  value!(
    vec![Symbol::Condition(Condition::Carry)],
    alt!(tag_no_case!("C.") | tag_no_case!("<"))
  ) |
  value!(
    vec![Symbol::Mode(Mode::Negate), Symbol::Condition(Condition::Carry)],
    alt!(tag_no_case!("C!") | tag_no_case!(">="))
  ) |
  value!(
    vec![Symbol::Condition(Condition::CarryAndNotZero)],
    tag_no_case!(">")
  ) |
  value!(
    vec![Symbol::Mode(Mode::Negate), Symbol::Condition(Condition::CarryAndNotZero)],
    tag_no_case!("<=")
  ) |
  value!(
    vec![Symbol::Condition(Condition::Overflow)],
    alt!(tag_no_case!("V.") | tag_no_case!("Lt"))
  ) |
  value!(
    vec![Symbol::Mode(Mode::Negate), Symbol::Condition(Condition::Overflow)],
    alt!(tag_no_case!("V!") | tag_no_case!("Ge"))
  ) |
  value!(
    vec![Symbol::Condition(Condition::OverflowAndNotZero)],
    tag_no_case!("Gt")
  ) |
  value!(
    vec![Symbol::Mode(Mode::Negate), Symbol::Condition(Condition::OverflowAndNotZero)],
    tag_no_case!("Le")
  )
));

named!(op(CompleteStr) -> Vec<Symbol>, alt!(
  value!(vec![Symbol::Op(Op::Ld)],   tag_no_case!("LD")) |
  value!(vec![Symbol::Op(Op::Add)],  tag_no_case!("ADD")) |
  value!(vec![Symbol::Op(Op::Sub)],  tag_no_case!("SUB")) |
  value!(vec![Symbol::Op(Op::Sbn)],  tag_no_case!("SBN")) |
  value!(vec![Symbol::Op(Op::Cmp)],  tag_no_case!("CMP")) |
  value!(vec![Symbol::Op(Op::Cpn)],  tag_no_case!("CPN")) |
  value!(vec![Symbol::Op(Op::And)],  tag_no_case!("AND")) |
  value!(vec![Symbol::Op(Op::Or)],   tag_no_case!("OR")) |
  value!(vec![Symbol::Op(Op::Xor)],  tag_no_case!("XOR")) |
  value!(vec![Symbol::Op(Op::Jmp)],  tag_no_case!("JMP")) |
  value!(vec![Symbol::Op(Op::Test)], tag_no_case!("TEST")) |
  value!(vec![Symbol::Op(Op::Set)],  tag_no_case!("SET")) |
  value!(vec![Symbol::Op(Op::Int)],  tag_no_case!("INT")) |
  value!(vec![Symbol::Op(Op::Brk)],  tag_no_case!("BRK")) |
  value!(vec![Symbol::Op(Op::Hlt)],  tag_no_case!("HLT")) |
  value!(vec![Symbol::Op(Op::Nop)],  tag_no_case!("NOP")) |
  do_parse!(
    o: alt!(
      value!(Op::Put, tag_no_case!("PUT")) |
      value!(Op::Pop, tag_no_case!("POP"))
    ) >>
    s: opt!(alt!(
      value!(Register::StackZero, one_of!("sS0")) |
      value!(Register::StackOne,  one_of!("dD1"))
    )) >>
    (match s {
      None => vec![Symbol::Op(o), Symbol::Register(Register::StackZero)],
      Some(s) => vec![Symbol::Op(o), Symbol::Register(s)],
    })
  )
));

named!(label(CompleteStr) -> Vec<Symbol>, terminated!(alt!(
  map!(recognize!(pair!(alpha1, alphanumeric0)), |v| vec![Symbol::Label(&v)]) |
  map!(take_while_m_n!(1, 1, call!(|c: char| c.is_digit(10))), |v| vec![Symbol::Label(&v)])
), one_of!(":")));

named!(token(CompleteStr) -> Vec<Symbol>, alt!(
  label | immediate | offset | indexed | stack_offset |
  indirect | variable | stack |
  map!(
    pair!(opt!(condition), op),
    |(mut c, mut o)| match c {
      None => o,
      Some(mut c) => { c.append(&mut o); c },
    }
  ) |
  map!(alt!(
    register | flags_register | stack_register | program_register |
    byte | word
  ), |v| vec![v])
));

named!(line(CompleteStr) -> Vec<Symbol>, map!(
  many0!(alt!(
    token |
    delimited!(one_of!(", \t"), token, one_of!(", \t")) |
    preceded!(one_of!(", \t"), token) |
    terminated!(token, one_of!(", \t"))
  )),
  |l| l.into_iter().flatten().collect::<Vec<Symbol>>()
));

named!(parser(CompleteStr) -> Vec<Symbol>, map!(
  separated_list!(line_ending, line),
  |l| l.into_iter().flatten().collect::<Vec<Symbol>>()
));

// TODO DIRECTIVES
// TODO "b"
// TODO make relative labels work right

pub fn parse(file: &str) -> Vec<u8> {
  let symbols: HashMap<&str, u16> = HashMap::new();
  let rel_symbols: HashMap<&str, HashMap<u16, u16>> = HashMap::new();

  let (remaining, parsed) = parser(CompleteStr("V.JMP 255")).unwrap();

  println!("\n{:?}", parsed);
  println!("\n{:?}", remaining);

  println!("");
  panic!("End.");

  // /\*(?:\+(\d+))?/ is a special symbol

  // first pass, find and set labels

  // second pass, assemble

  vec![]
}






// Directives: begin with #
// Label: Defined by ending with :
// Label: Can't be a register name.
// Label: Used normally.
// RelLabel: Defined by single decimal digit ending with :
// RelLabel: Used with either + or - postfix
// No need for whitespace between conditon and mnemonic. (Everything else needs whitespace.)
// Whitespace between operands can be replaced with a ',' if desired.
// Comments begin with // and go until the next '\n'
