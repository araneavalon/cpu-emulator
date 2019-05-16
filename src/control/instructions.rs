
use std::fmt;

use super::microcode::MicrocodeArray;
use super::control::{Control, Register};
use crate::error::{Error, Result};


#[derive(Clone)]
pub struct Iter {
  name: &'static str,
  op: u16,
  last_index: Option<usize>,
  iter: std::iter::Peekable<std::vec::IntoIter<(usize, Control)>>,
}
impl Iter {
  fn new(name: &'static str, op: u16, vec: Vec<(usize, Control)>) -> Iter {
    Iter { name, op, last_index: None, iter: vec.into_iter().peekable() }
  }

  pub fn peek(&mut self) -> Option<&(usize, Control)> {
    self.iter.peek()
  }
}
impl Iterator for Iter {
  type Item = Control;

  fn next(&mut self) -> Option<Control> {
    match self.iter.next() {
      Some((index, c)) => {
        self.last_index = Some(index);
        Some(c)
      },
      None => {
        self.last_index = None;
        None
      },
    }
  }
}
impl fmt::Debug for Iter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.last_index {
        Some(index) => write!(f, "Instruction(0x{:04X} => {:2}: {})", self.op, index, self.name),
        None => write!(f, "Fetch()"),
    }
  }
}


#[derive(Debug)]
pub enum Branch {
  None,
  Near,
  Far,
  Interrupt,
}
impl Branch {
  fn mask(&self) -> Option<(bool, u16)> {
    match self {
      Branch::None => None,
      Branch::Near => Some((false, 0x0400)),
      Branch::Far => Some((false, 0x1000)),
      Branch::Interrupt => Some((true, 0x0000)),
    }
  }
}


#[derive(Debug)]
pub enum Instruction {
  Stack(Stack),
  Normal(Normal),
  A(Argument),
}
impl Instruction {
  pub fn stack(name: &'static str, base: usize) -> Instruction {
    Instruction::Stack(Stack::new(name, base))
  }
  pub fn normal(name: &'static str, branch: Branch, microcode: Vec<usize>) -> Instruction {
    Instruction::Normal(Normal::new(name, branch, microcode))
  }
  pub fn a(branch: Branch, microcode: [(&'static str, Vec<usize>); 5]) -> Instruction {
    Instruction::A(Argument::new(branch, microcode))
  }

  pub fn decode(&self, microcode: &MicrocodeArray, op: u16) -> Result<Iter> {
    match self {
      Instruction::Stack(v)  => v.decode(microcode, op),
      Instruction::Normal(v) => v.decode(microcode, op),
      Instruction::A(v)      => v.decode(microcode, op),
    }
  }
}


#[derive(Debug)]
pub struct Stack {
  name: &'static str,
  base: usize,
}
impl Stack {
  pub fn new(name: &'static str, base: usize) -> Stack {
    Stack { name, base }
  }

  pub fn decode(&self, microcode: &MicrocodeArray, op: u16) -> Result<Iter> {
    let mut out = Vec::new();

    let direction = (op & 0x0400) != 0;
    let registers = ((op & 0x0800) >> 2) | (op & 0x01FF);
    let base = microcode[self.base].decode(op, None)?;

    for cycle in 0..10 {
      let bit = if direction { 9 - cycle } else { cycle };
      if (registers & (1 << bit)) != 0 {
        let mut c = base.clone();
        match bit {
          0 => { if direction { c.register.load = Register::Zero; } else { c.register.out = Register::Zero; } },
          1 => { if direction { c.register.load = Register::One; } else { c.register.out = Register::One; } },
          2 => { if direction { c.register.load = Register::Two; } else { c.register.out = Register::Two; } },
          3 => { if direction { c.register.load = Register::Three; } else { c.register.out = Register::Three; } },
          4 => { if direction { c.register.load = Register::Four; } else { c.register.out = Register::Four; } },
          5 => { if direction { c.register.load = Register::Five; } else { c.register.out = Register::Five; } },
          6 => { if direction { c.register.load = Register::Six; } else { c.register.out = Register::Six; } },
          7 => { if direction { c.register.load = Register::Seven; } else { c.register.out = Register::Seven; } },
          8 => { if direction { c.flags.load = true; } else { c.flags.out = true; } },
          9 => { if direction { c.pc.load = true; } else { c.lr.out = true; } },
          _ => return Err(Error::Impossible(op, "Encountered an invalid bit during stack instruction decode.")),
        }
        out.push((self.base, c));
      }
    }

    Ok(Iter::new(self.name, op, out))
  }
}


#[derive(Debug)]
pub struct Normal {
  name: &'static str,
  branch: Branch,
  microcode: Vec<usize>,
}
impl Normal {
  pub fn new(name: &'static str, branch: Branch, microcode: Vec<usize>) -> Normal {
    Normal { name, branch, microcode }
  }

  pub fn decode(&self, microcode: &MicrocodeArray, op: u16) -> Result<Iter> {
    let branch = self.branch.mask();
    let vec = self.microcode.iter()
      .map(|index| Ok((*index, microcode[*index].decode(op, branch)?)))
      .collect::<Result<Vec<(usize, Control)>>>()?;
    Ok(Iter::new(self.name, op, vec))
  }
}


const ARGUMENT_DECODE_TABLE: [usize; 16] = [
  0, 1, 0, 1, 2, 3, 2, 3,
  4, 4, 4, 4, 4, 4, 4, 4,
];

#[derive(Debug)]
pub struct Argument {
  branch: Branch,
  microcode: [(&'static str, Vec<usize>); 5],
}
impl Argument {
  pub fn new(branch: Branch, microcode: [(&'static str, Vec<usize>); 5]) -> Argument {
    Argument { branch, microcode }
  }

  pub fn decode(&self, microcode: &MicrocodeArray, op: u16) -> Result<Iter> {
    let mode = ARGUMENT_DECODE_TABLE[((op as usize) & 0x03C0) >> 6];
    let branch = self.branch.mask();
    let vec = self.microcode[mode].1.iter()
      .map(|index| Ok((*index, microcode[*index].decode(op, branch)?)))
      .collect::<Result<Vec<(usize, Control)>>>()?;
    Ok(Iter::new(self.microcode[mode].0, op, vec))
  }
}


const DECODE_TABLE: [usize; 512] = [
  22, 22, 22, 22, 22, 22, 22, 22, 21, 21, 21, 21, 21, 21, 21, 21,
  16, 17, 18, 19, 20, 20, 20, 20, 16, 17, 18, 19, 20, 20, 20, 20,
  11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11,
  11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11,
   3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,
   3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,
   3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,
   3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,  3,
  10, 10,  9,  9, 10, 10,  9,  9, 10, 10,  9,  9, 10, 10,  9,  9,
  10, 10,  9,  9, 10, 10,  9,  9, 10, 10,  9,  9, 10, 10,  9,  9,
   6,  6,  6,  6,  6,  6,  6,  6,  6,  6,  6,  6,  6,  6,  6,  6,
   6,  6,  6,  6,  6,  6,  6,  6,  6,  6,  6,  6,  6,  6,  6,  6,
  14, 14, 14, 14, 14, 14, 14, 14, 15, 15, 15, 15, 15, 15, 15, 15,
  13, 13, 13, 13, 13, 13, 13, 13, 12, 12, 12, 12, 12, 12, 12, 12,
   2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,
   2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,
   5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,
   5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,
   5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,  5,
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
   8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,
   8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,
   8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,
   8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,  8,
   4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,
   4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,
   4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,
   1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,
   7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,
   7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,
   7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,
   7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,  7,
];


pub struct Instructions {
  fetch: Control,
  init: Iter,
  instructions: [Instruction; 23],
}

impl Instructions {
  pub fn new(microcode: &MicrocodeArray) -> Result<Instructions> {
    Ok(Instructions {
      fetch: microcode[0].decode(0x0000, None)?, // Opcode doesn't matter for fetch.
      init: Instructions::init_iter(microcode)?,
      instructions: Instructions::array(),
    })
  }

  fn init_iter(microcode: &MicrocodeArray) -> Result<Iter> {
    Instruction::normal("INIT", Branch::None, vec![45, 43]).decode(microcode, 0x0000)
  }

  fn array() -> [Instruction; 23] {
    [
      Instruction::a(Branch::None, [ // 0 LD r,a
        ("LD r,r",      vec![1]),
        ("LD r,(r)",    vec![2, 3]),
        ("LD r,word",   vec![4]),
        ("LD r,(word)", vec![5, 3]),
        ("LD r,(r+r)",  vec![6, 7, 8, 3]),
      ]),
      Instruction::normal("LD r,b", Branch::None, vec![9]), // 1 LD r,b
      Instruction::normal("LD r,(u)", Branch::None, vec![10, 11]), // 2 LD r,(u)

      Instruction::a(Branch::None, [ // 3 OP r,a
        ("OP r,r",      vec![12, 13, 14]),
        ("OP r,(r)",    vec![12, 2, 15, 14]),
        ("OP r,word",   vec![12, 16, 14]),
        ("OP r,(word)", vec![12, 5, 15, 14]),
        ("OP r,(r+r)",  vec![6, 7, 8, 12, 15, 14]),
      ]),
      Instruction::normal("OP r,b", Branch::None, vec![17, 18, 19]), // 4 OP r,b
      Instruction::normal("OP r,(u)", Branch::None, vec![17, 10, 20, 19]), // 5 OP r,(u)

      Instruction::a(Branch::Near, [ // 6 JMl a
        ("JMl r",      vec![21]),
        ("JMl (r)",    vec![2, 22]),
        ("JMl word",   vec![23]),
        ("JMl (word)", vec![5, 22]),
        ("JMl (r+r)",  vec![6, 7, 8, 22]),
      ]),
      Instruction::normal("JMl b", Branch::Far, vec![24, 25, 26]), // 7 JMl b
      Instruction::normal("JMl (u)", Branch::Far, vec![10, 22]), // 8 JMl (u)

      Instruction::normal("RET", Branch::Near, vec![27]), // 9 RET
      Instruction::normal("RETs", Branch::Near, vec![28]), // 10 RETs

      Instruction::stack("PUT/POP", 29), // 11 PUTs/POPs

      Instruction::normal("SET F", Branch::None, vec![30, 31, 32]), // 12 SET F,b,v
      Instruction::normal("SET r", Branch::None, vec![12, 31, 33]), // 13 SET r,b,v
      Instruction::normal("TEST r", Branch::None, vec![12, /*31,*/ 34]), // 14 TEST r,b

      Instruction::normal("UOP r", Branch::None, vec![12, 35]), // 15 UOP r

      Instruction::normal("LD x,r",      Branch::None, vec![36]), // 16 LD x,r
      Instruction::normal("LD x,(r)",    Branch::None, vec![37, 38]), // 17 LD x,(r)
      Instruction::normal("LD x,word",   Branch::None, vec![39]), // 18 LD x,word
      Instruction::normal("LD x,(word)", Branch::None, vec![5, 38]), // 19 LD x,(word)
      Instruction::normal("LD x,(r+r)",  Branch::None, vec![40, 7, 8, 38]), // 20 LD x,(r+r)

      Instruction::normal("INT i", Branch::Interrupt, vec![41, 42, 43]), // 21 INT
      Instruction::normal("NOP", Branch::None, vec![44]), // 22 NOP
    ]
  }

  pub fn fetch(&self) -> Control {
    self.fetch
  }

  pub fn init(&self) -> Iter {
    self.init.clone()
  }

  pub fn interrupt(&self, microcode: &MicrocodeArray, interrupt: u16) -> Result<(u16, Iter)> {
    let op = 0x0400 | (interrupt << 3);
    Ok((op, self.instructions[21].decode(&microcode, op)?))
  }

  pub fn decode(&self, microcode: &MicrocodeArray, op: u16) -> Result<Iter> {
    self.instructions[DECODE_TABLE[(op as usize) >> 7]].decode(&microcode, op)
  }
}
