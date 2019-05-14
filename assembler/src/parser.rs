
use nom::{
  types::CompleteStr,
  IResult,
  InputTakeAtPosition,

  rest,
  hex_digit1,
  digit1,
};

use super::symbols::*;
use super::error::{
  Error,
  Result,
};


fn sp0(input: CompleteStr) -> IResult<CompleteStr, CompleteStr> {
  input.split_at_position(|item| (item != ' ') && (item != '\t'))
}

fn sp1(input: CompleteStr) -> IResult<CompleteStr, CompleteStr> {
  input.split_at_position1(|item| (item != ' ') && (item != '\t'), nom::ErrorKind::Custom(0x01))
}

fn label_start(input: CompleteStr) -> IResult<CompleteStr, CompleteStr> {
  input.split_at_position1(|item| !item.is_alphabetic() && (item != '_'), nom::ErrorKind::Custom(0x02))
}
fn label_char(input: CompleteStr) -> IResult<CompleteStr, CompleteStr> {
  input.split_at_position(|item| !item.is_alphanumeric() && (item != '_'))
}
fn label_str(input: CompleteStr) -> IResult<CompleteStr, CompleteStr> {
  match recognize!(input, pair!(label_start, label_char)) {
    Err(error) => Err(error),
    Ok((i, o)) => {
      match any_register(o) {
        Ok((i, _)) if i.len() == 0 => Err(nom::Err::Error(error_position!(input, nom::ErrorKind::Custom(0x03)))),
        _ => Ok((i, o)),
      }
    },
  }
}

named!(relative_str(CompleteStr) -> CompleteStr,
  take_while_m_n!(1, 1, call!(|c: char| c.is_digit(10) | c.is_lowercase()))
);

named!(sep1(CompleteStr) -> CompleteStr,
  recognize!(delimited!(sp0, one_of!(","), sp0))
);

named!(number(CompleteStr) -> Value, map!(
  alt!(
    map_res!(preceded!(tag!("0x"), hex_digit1), |v: CompleteStr| u16::from_str_radix(&v, 16)) |
    map_res!(preceded!(opt!(one_of!("+")), digit1), |v: CompleteStr| u16::from_str_radix(&v, 10)) |
    map_res!(
      recognize!(preceded!(one_of!("-"), digit1)),
      |v: CompleteStr| match i16::from_str_radix(&v, 10) {
        Err(e) => Err(e),
        Ok(v) => Ok(v as u16),
      }
    ) |
    map_opt!(
      delimited!(one_of!("'"), take_while_m_n!(1, 2, |c| c != '\''), one_of!("'")),
      |v: CompleteStr| match v.as_bytes() {
        [l] => Some(*l as u16),
        [h, l] => Some(((*h as u16) << 8) | (*l as u16)),
        _ => None,
      }
    )
  ),
  |v| Value::Const(v)
));

named!(relative(CompleteStr) -> Value, map!(
  delimited!(
    one_of!("("),
    pair!(relative_str, one_of!("+-")),
    one_of!(")")
  ),
  |(label, direction)| match direction {
    '+' => Value::Relative(true, &label),
    '-' => Value::Relative(false, &label),
    _ => panic!("Should be unable to get this value.")
  }
));

named!(label(CompleteStr) -> Value, alt!(
  value!(Value::Star, one_of!("*")) |
  map!(label_str, |v| Value::Label(&v))
));

named!(value(CompleteStr) -> Value, alt!(relative | label | number));

named!(expression(CompleteStr) -> Expression, alt!(
  do_parse!(
    a: value >> sp0 >>
    o: one_of!("+-") >> sp0 >>
    b: value >>
    (match o {
      '+' => Expression::Add(a, b),
      '-' => Expression::Sub(a, b),
      _ => panic!("Impossible."),
    })
  ) |
  map!(value, |v| Expression::Value(v))
));

named!(string(CompleteStr) -> Vec<Value>, map!(
  delimited!(one_of!("\""), is_not!("\""), one_of!("\"")),
  |v| v.as_bytes().into_iter().map(|v| Value::Const(*v as u16)).collect::<Vec<Value>>()
));

named!(register(CompleteStr) -> Register, alt!(
  value!(Register::Zero,  alt!(tag_no_case!("R0") | tag_no_case!("A"))) |
  value!(Register::One,   alt!(tag_no_case!("R1") | tag_no_case!("B"))) |
  value!(Register::Two,   alt!(tag_no_case!("R2") | tag_no_case!("C"))) |
  value!(Register::Three, alt!(tag_no_case!("R3") | tag_no_case!("D"))) |
  value!(Register::Four,  alt!(tag_no_case!("R4") | tag_no_case!("E"))) |
  value!(Register::Five,  alt!(tag_no_case!("R5") | tag_no_case!("X"))) |
  value!(Register::Six,   alt!(tag_no_case!("R6") | tag_no_case!("Y"))) |
  value!(Register::Seven, alt!(tag_no_case!("R7") | tag_no_case!("Z")))
));

named!(flags_register(CompleteStr) -> CompleteStr, tag_no_case!("F"));

named!(stack_register(CompleteStr) -> StackRegister, alt!(
  value!(StackRegister::Zero, alt!(tag_no_case!("S0") | tag_no_case!("SR"))) |
  value!(StackRegister::One,  alt!(tag_no_case!("S1") | tag_no_case!("SD")))
));

named!(program_counter(CompleteStr) -> ProgramRegister,
  value!(ProgramRegister::ProgramCounter, tag_no_case!("PC"))
);
named!(link_register(CompleteStr) -> ProgramRegister,
  value!(ProgramRegister::LinkRegister, tag_no_case!("LR"))
);

named!(any_register(CompleteStr) -> AnyRegister, alt!(
  map!(register, |r| AnyRegister::Register(r)) |
  map!(stack_register, |s| AnyRegister::Stack(s)) |
  map!(alt!(program_counter | link_register), |p| AnyRegister::Program(p))
));

named!(indexed(CompleteStr) -> Argument, do_parse!(
  one_of!("(") >> sp0 >>
  b: register >> sp0 >>
  one_of!("+") >> sp0 >>
  i: register >> sp0 >>
  one_of!(")") >>
  (Argument::Indexed(b, i))
));

named!(indirect(CompleteStr) -> Argument, do_parse!(
  one_of!("(") >> sp0 >>
  r: register >> sp0 >>
  one_of!(")") >>
  (Argument::Indirect(r))
));

named!(variable(CompleteStr) -> Argument, do_parse!(
  one_of!("(") >> sp0 >>
  w: expression >> sp0 >>
  one_of!(")") >>
  (Argument::Variable(w))
));

named!(argument(CompleteStr) -> Argument, alt!(
  indexed |
  variable |
  indirect |
  map!(expression, |e| Argument::Constant(e)) |
  map!(register, |r| Argument::Direct(r))
));

fn stack_program_register(input: CompleteStr, direction: bool) -> IResult<CompleteStr, ProgramRegister> {
  if direction {
    program_counter(input)
  } else {
    link_register(input)
  }
}
fn stack_argument(input: CompleteStr, direction: bool) -> IResult<CompleteStr, [bool; 10]> {
  do_parse!(input,
    one_of!("[") >> sp0 >>
    registers: separated_list!(sep1, alt!(
      map!(call!(stack_program_register, direction), |_| 9) |
      map!(flags_register, |_| 8) |
      map!(register, |r| r as usize)
    )) >> sp0 >>
    one_of!("]") >>
    ({
      let mut out = [false; 10];
      for index in registers {
        out[index] = true;
      }
      out
    })
  )
}

named!(condition(CompleteStr) -> Condition, alt!(
  value!(Condition::Always(true),              tag_no_case!("!")) |
  value!(Condition::Zero(false),               alt!(tag_no_case!("Z.") | tag_no_case!("E."))) |
  value!(Condition::Zero(true),                alt!(tag_no_case!("Z!") | tag_no_case!("E!"))) |
  value!(Condition::Sign(false),               alt!(tag_no_case!("N.") | tag_no_case!("P!"))) |
  value!(Condition::Sign(true),                alt!(tag_no_case!("N!") | tag_no_case!("P."))) |
  value!(Condition::Carry(false),              alt!(tag_no_case!("C.") | tag_no_case!("<"))) |
  value!(Condition::Carry(true),               alt!(tag_no_case!("C!") | tag_no_case!(">="))) |
  value!(Condition::CarryAndNotZero(false),    tag_no_case!(">")) |
  value!(Condition::CarryAndNotZero(true),     tag_no_case!("<=")) |
  value!(Condition::Overflow(false),           alt!(tag_no_case!("V.") | tag_no_case!("Lt"))) |
  value!(Condition::Overflow(true),            alt!(tag_no_case!("V!") | tag_no_case!("Ge"))) |
  value!(Condition::OverflowAndNotZero(false), tag_no_case!("Gt")) |
  value!(Condition::OverflowAndNotZero(true),  tag_no_case!("Le"))
));

named!(alu_op(CompleteStr) -> Op, do_parse!(
  op: alt!(
    value!(AluOp::Add, tag_no_case!("ADD")) |
    value!(AluOp::Sub, tag_no_case!("SUB")) |
    value!(AluOp::Sbn, tag_no_case!("SBN")) |
    value!(AluOp::Cmp, tag_no_case!("CMP")) |
    value!(AluOp::Cpn, tag_no_case!("CPN")) |
    value!(AluOp::And, tag_no_case!("AND")) |
    value!(AluOp::Or,  tag_no_case!("OR"))  |
    value!(AluOp::Xor, tag_no_case!("XOR"))
  ) >> sp1 >>
  r: register >> sep1 >>
  a: argument >>
  (Op::Alu(op, r, a))
));

named!(unary_op(CompleteStr) -> Op, do_parse!(
  op: alt!(
    value!(UnaryOp::Not, tag_no_case!("NOT")) |
    value!(UnaryOp::Neg, tag_no_case!("NEG")) |
    value!(UnaryOp::Sl,  tag_no_case!("SL")) |
    value!(UnaryOp::Asr, tag_no_case!("ASR")) |
    value!(UnaryOp::Lsr, tag_no_case!("LSR"))
  ) >> sp1 >>
  r: register >>
  (Op::Unary(op, r))
));

named!(jmp_op(CompleteStr) -> Op, alt!(
  do_parse!(
    c: opt!(condition) >>
    l: alt!(
      value!(false, tag_no_case!("JMP")) |
      value!(true, tag_no_case!("JML")) |
      value!(true, tag_no_case!("JMPL"))
    ) >> sp1 >>
    a: alt!(
      map!(argument, |a| JumpArgument::Argument(a)) |
      map!(link_register, |_| JumpArgument::LinkRegister)
    ) >>
    (match c {
      Some(c) => Op::Jump(c, l, a),
      None => Op::Jump(Condition::Always(false), l, a),
    })
  ) |
  do_parse!(
    c: opt!(condition) >>
    l: alt!(
      value!(false, tag_no_case!("POP")) |
      value!(true,  tag_no_case!("POPL"))
    ) >>
    s: opt!(alt!(
      value!(StackRegister::Zero, one_of!("sS0")) |
      value!(StackRegister::One, one_of!("dD1"))
    )) >> sp1 >>
    program_counter >>
    (match s {
      Some(s) => Op::Jump(Condition::from(c), l, JumpArgument::Stack(s)),
      None => Op::Jump(Condition::from(c), l, JumpArgument::Stack(StackRegister::Zero)),
    })
  ) |
  do_parse!(
    c: opt!(condition) >>
    l: alt!(
      value!(false, tag_no_case!("RET")) |
      value!(true,  tag_no_case!("RTL"))
    ) >>
    s: opt!(alt!(
      value!(StackRegister::Zero, one_of!("sS0")) |
      value!(StackRegister::One, one_of!("dD1"))
    )) >>
    (match s {
      Some(s) => Op::Jump(Condition::from(c), l, JumpArgument::Stack(s)),
      None => Op::Jump(Condition::from(c), l, JumpArgument::LinkRegister),
    })
  )
));

fn op(input: CompleteStr, index: usize) -> IResult<CompleteStr, Symbol> {
  map!(input, alt!(
    alu_op |
    unary_op |
    do_parse!(
      v: alt!(
        value!(Expression::Value(Value::Const(0x0001)), tag_no_case!("INC")) |
        value!(Expression::Value(Value::Const(0xFFFF)), tag_no_case!("DEC"))
      ) >> sp1 >>
      r: register >>
      (Op::Alu(AluOp::Add, r, Argument::Constant(v)))
    ) |
    do_parse!(tag_no_case!("TEST") >> sp1 >> r: register >> sep1 >> m: value >> (Op::Test(r, m))) |
    do_parse!(
      tag_no_case!("SET") >> sp1 >>
      r: register >> sep1 >>
      m: value >> sep1 >>
      v: one_of!("01") >>
      (match v {
        '0' => Op::Set(r, m, false),
        '1' => Op::Set(r, m, true),
        _ => panic!("Impossible"),
      })
    ) |
    do_parse!(
      tag_no_case!("SET") >> sp1 >>
      flags_register >> sep1 >>
      m: value >> sep1 >>
      v: one_of!("01") >>
      (match v {
        '0' => Op::SetFlags(m, false),
        '1' => Op::SetFlags(m, true),
        _ => panic!("Impossible"),
      })
    ) |
    do_parse!(
      tag_no_case!("LD") >> sp1 >>
      r: any_register >> sep1 >>
      a: argument >>
      (Op::Load(true, r, a))
    ) |
    do_parse!(
      tag_no_case!("LD") >> sp1 >>
      a: argument >> sep1 >>
      r: any_register >>
      (Op::Load(false, r, a))
    ) |
    jmp_op |
    do_parse!(
      d: alt!(
        value!(true, tag_no_case!("POP")) |
        value!(false, tag_no_case!("PUT"))
      ) >>
      s: opt!(alt!(
        value!(StackRegister::Zero, one_of!("sS0")) |
        value!(StackRegister::One, one_of!("dD1"))
      )) >> sp1 >>
      r: call!(stack_argument, d) >>
      (match s {
        None => Op::Stack(d, StackRegister::Zero, r),
        Some(s) => Op::Stack(d, s, r),
      })
    ) |
    do_parse!(
      h: alt!(
        value!(true, tag_no_case!("BRK")) |
        value!(false, tag_no_case!("INT"))
      ) >> sp1 >>
      v: value >>
      (Op::Interrupt(h, v))
    ) |
    value!(Op::Nop(true), tag_no_case!("HLT")) |
    value!(Op::Nop(false), tag_no_case!("NOP"))
  ), |op| Symbol::Op(index, op))
}

fn relative_define(input: CompleteStr, index: usize) -> IResult<CompleteStr, Symbol> {
  map!(input, relative_str, |v| Symbol::Relative(index, &v))
}

fn label_define(input: CompleteStr, index: usize) -> IResult<CompleteStr, Symbol> {
  map!(input, label_str, |v| Symbol::Label(index, &v))
}

fn comment(input: CompleteStr, index: usize) -> IResult<CompleteStr, Symbol> {
  do_parse!(input,
    tag!("//") >>
    sp0 >>
    c: rest >>
    (Symbol::Comment(index, &c))
  )
}

fn import_line(input: CompleteStr, index: usize) -> IResult<CompleteStr, Vec<Symbol>> {
  do_parse!(input,
    tag_no_case!("#import") >> sp1 >>
    p: delimited!(one_of!("\""), is_not!("\""), one_of!("\"")) >> sp0 >>
    c: opt!(call!(comment, index)) >>
    (match c {
      Some(c) => vec![Symbol::Import(index, &p), c],
      None => vec![Symbol::Import(index, &p)],
    })
  )
}

fn define_line(input: CompleteStr, index: usize) -> IResult<CompleteStr, Vec<Symbol>> {
  alt!(input,
    do_parse!(
      tag_no_case!("#define") >> sp1 >>
      l: label_str >> sp0 >>
      one_of!("=") >> sp0 >>
      e: expression >> sp0 >>
      c: opt!(call!(comment, index)) >>
      (match c {
        Some(c) => vec![Symbol::Define(index, &l, e), c],
        None => vec![Symbol::Define(index, &l, e)],
      })
    ) |
    do_parse!(
      tag_no_case!("#define") >> sp1 >>
      one_of!("*") >> sp0 >>
      one_of!("=") >> sp0 >>
      v: number >> sp0 >>
      c: opt!(call!(comment, index)) >>
      (match (c, v) {
        (Some(c), Value::Const(v)) => vec![Symbol::Star(index, v), c],
        (None, Value::Const(v)) => vec![Symbol::Star(index, v)],
        _ => panic!("Impossible value."),
      })
    )
  )
}

fn word_line(input: CompleteStr, index: usize) -> IResult<CompleteStr, Vec<Symbol>> {
  do_parse!(input,
    l: opt!(terminated!(alt!(
      call!(relative_define, index) |
      call!(label_define, index)
    ), one_of!(":"))) >> sp0 >>
    tag_no_case!("#word") >> sp1 >>
    w: separated_list!(sep1, alt!(
      map!(expression, |w| vec![w]) |
      map!(string, |s| s.into_iter().map(|v| Expression::Value(v)).collect::<Vec<Expression>>())
    )) >> sp0 >>
    opt!(call!(comment, index)) >>
    ({
      let mut words = w.into_iter()
        .flatten()
        .map(|e| Symbol::Word(index, e))
        .collect::<Vec<Symbol>>();
      if let Some(l) = l {
        words.insert(0, l);
      }
      words
    })
  )
}

fn op_line(input: CompleteStr, index: usize) -> IResult<CompleteStr, Vec<Symbol>> {
  do_parse!(input,
    l: opt!(terminated!(alt!(
      call!(relative_define, index) |
      call!(label_define, index)
    ), one_of!(":"))) >> sp0 >>
    o: opt!(call!(op, index)) >> sp0 >>
    c: opt!(call!(comment, index)) >>
    (vec![l, o, c].into_iter().filter_map(|v| v).collect::<Vec<Symbol>>())
  )
}

fn parse_line(input: CompleteStr, index: usize) -> IResult<CompleteStr, Vec<Symbol>> {
  alt!(input,
    call!(import_line, index) |
    call!(define_line, index) |
    call!(word_line, index) |
    call!(op_line, index)
  )
}


pub fn parse(input: &str) -> Result<Vec<Symbol>> {
  let mut out = Vec::new();
  for (index, line) in input.lines().enumerate() {
    match parse_line(CompleteStr(line), index + 1) {
      Err(e) => return Err(Error::parser(index + 1, e)),
      Ok((i, _)) if i.len() > 0 => return Err(Error::parser(index + 1,
        nom::Err::Error(error_position!(i, nom::ErrorKind::Custom(0x00))))),
      Ok((_, mut o)) => out.append(&mut o),
    }
  }
  Ok(out)
}
