
use crate::error::{Error, Result};
use super::control::{self, Control};


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum AddressSelect {
  A,
  ProgramCounter,
  S,
}

impl AddressSelect {
  fn decode(&self, op: u16, c: &mut Control) {
    c.address = match self {
      AddressSelect::S if (op & 0x0200) == 0 => control::Address::StackZero,
      AddressSelect::S => control::Address::StackOne,
      AddressSelect::A => control::Address::A,
      AddressSelect::ProgramCounter => control::Address::ProgramCounter,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DataSelect {
  None,

  // Load/Out
  RegisterZero,
  X, // PC, LR, S0, S1
  ProgramCounter,
  LinkRegister,
  F,
  Memory,

  // Out
  RegisterOne,
  RegisterTwo,
  Alu,
  SignedByte,
  UnsignedByte,
  Bitmask,
  Interrupt,
  Startup,

  // Load
  T(bool),
  A,
  I,
}

macro_rules! bi {
  ( $c:expr, $d:ident, $v:expr ) => {
    { if $d { $c.load = $v; } else { $c.out = $v; } }
  };
}
macro_rules! assert_read {
  ($d:ident, $op:ident, $m:expr) => {
    { if $d { return Err(Error::InvalidWrite($op, $m)) } }
  };
}
macro_rules! assert_write {
  ($d:ident, $op:ident, $m:expr) => {
    { if !$d { return Err(Error::InvalidRead($op, $m)) } }
  };
}

impl DataSelect {
  fn decode(&self, op: u16, d: bool, c: &mut Control) -> Result<()> {
    match self {
      DataSelect::None => (),

      DataSelect::RegisterZero => {
        bi!(c.register, d, DataSelect::parse_register(op, 0)?);
      },
      DataSelect::RegisterOne => {
        assert_read!(d, op, "Can not write to register offset 3.");
        c.register.out = DataSelect::parse_register(op, 3)?;
      },
      DataSelect::RegisterTwo => {
        assert_read!(d, op, "Can not write to register offset 6.");
        c.register.out = DataSelect::parse_register(op, 6)?;
      },

      DataSelect::T(s) => {
        assert_write!(d, op, "Can not write to Alu T Registers.");
        c.alu.t[*s as usize].load = true;
      },
      DataSelect::Alu => {
        assert_read!(d, op, "Can not write to Alu Output.");
        c.alu.out = true;
      },
      DataSelect::F => {
        bi!(c.flags, d, true);
      },

      DataSelect::ProgramCounter => {
        bi!(c.pc, d, true);
      },
      DataSelect::LinkRegister => {
        bi!(c.lr, d, true);
      },
      DataSelect::X => {
        match (op & 0x0018) >> 3 {
          0 => bi!(c.s[0], d, true),
          1 => bi!(c.s[1], d, true),
          2 => bi!(c.pc, d, true),
          3 => bi!(c.lr, d, true),
          value => return Err(Error::InvalidExtraRegister(op, value)),
        }
      },

      DataSelect::A => {
        assert_write!(d, op, "Can not write to Address Register.");
        c.a.load = true;
      },
      DataSelect::Memory => {
        bi!(c.memory, d, true);
      },

      DataSelect::I => {
        assert_write!(d, op, "Can not write to Instruction Register.");
        c.i.load = true;
      },
      DataSelect::SignedByte => {
        assert_read!(d, op, "Can not write to Signed Byte Register.");
        c.i.mode = control::IMode::SignedByte;
      },
      DataSelect::UnsignedByte => {
        assert_read!(d, op, "Can not write to Unsigned Byte Register.");
        c.i.mode = control::IMode::UnsignedByte;
      },
      DataSelect::Bitmask => {
        assert_read!(d, op, "Can not write to Bitmask Register.");
        c.i.mode = control::IMode::Bitmask;
      },
      DataSelect::Interrupt => {
        assert_read!(d, op, "Can not write to Interrupt Register.");
        c.i.mode = control::IMode::Interrupt;
      },
      DataSelect::Startup => {
        assert_read!(d, op, "Can not write to Init Address.");
        c.i.mode = control::IMode::Startup;
      }
    }
    Ok(())
  }

  fn parse_register(op: u16, offset: u16) -> Result<control::Register> {
    match (op >> offset) & 0x0007 {
      0x0 => Ok(control::Register::Zero),
      0x1 => Ok(control::Register::One),
      0x2 => Ok(control::Register::Two),
      0x3 => Ok(control::Register::Three),
      0x4 => Ok(control::Register::Four),
      0x5 => Ok(control::Register::Five),
      0x6 => Ok(control::Register::Six),
      0x7 => Ok(control::Register::Seven),
      value => Err(Error::InvalidRegister(op, offset, value)),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
  Const,
  Near,
  Far,
  Pop,
  Put,
}

impl Direction {
  fn parse(&self, op: u16, default: bool) -> bool {
    match self {
      Direction::Const => default,
      Direction::Near => default ^ ((op & 0x0400) == 0),
      Direction::Far => default ^ ((op & 0x0800) == 0),
      Direction::Pop => (op & 0x0800) != 0,
      Direction::Put => (op & 0x0800) == 0,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum AluMode {
  None,
  Unary,
  Short,
  Binary,
  Add,
  Test,
  Set,
}

impl AluMode {
  fn decode(&self, op: u16, c: &mut Control) -> Result<()> {
    match self {
      AluMode::None => (),

      AluMode::Unary => AluMode::decode_unary((op & 0x0038) >> 3, c)?,
      AluMode::Short => AluMode::decode_binary((op & 0x1800) >> 10, c)?,
      AluMode::Binary => AluMode::decode_binary((op & 0x1C00) >> 10, c)?,

      AluMode::Add => {
        c.alu.mode = control::AluMode::Add;
      },
      AluMode::Test => {
        c.alu.mode = control::AluMode::And;
        c.alu.out = false;
      },
      AluMode::Set => {
        if (op & 0x0080) != 0 {
          c.alu.mode = control::AluMode::Or;
        } else {
          c.alu.mode = control::AluMode::And;
          c.alu.t1_invert = true;
        }
      },
    }
    Ok(())
  }

  fn decode_binary(op: u16, c: &mut Control) -> Result<()> {
    match op {
      0 => c.alu.mode = control::AluMode::Add, // ADD
      1 => c.alu.mode = control::AluMode::And, // AND
      2 => {
        c.alu.mode = control::AluMode::Add; // CMP
        c.alu.t1_invert = true;
        c.alu.carry_invert = true;
        c.alu.out = false;
      },
      3 => {
        c.alu.mode = control::AluMode::Add; // SUB
        c.alu.t1_invert = true;
        c.alu.carry_invert = true;
      },
      4 => {
        c.alu.mode = control::AluMode::Add; // CPN
        c.alu.t1_invert = true;
        c.alu.carry_invert = true;
        if c.alu.t[0].load || c.alu.t[1].load {
          c.alu.t[0].load = !c.alu.t[0].load;
          c.alu.t[1].load = !c.alu.t[1].load;
        }
        c.alu.out = false;
      },
      5 => {
        c.alu.mode = control::AluMode::Add; // SBN
        c.alu.t1_invert = true;
        c.alu.carry_invert = true;
        if c.alu.t[0].load || c.alu.t[1].load {
          c.alu.t[0].load = !c.alu.t[0].load;
          c.alu.t[1].load = !c.alu.t[1].load;
        }
      },
      6 => c.alu.mode = control::AluMode::Or, // OR
      7 => c.alu.mode = control::AluMode::Xor, // XOR
      value => return Err(Error::InvalidBinaryOp(op, value)),
    }
    Ok(())
  }

  fn decode_unary(op: u16, c: &mut Control) -> Result<()> {
    match op {
      0b000 => { // NEG
        c.alu.mode = control::AluMode::Add;
        c.alu.t0_zero = true;
        c.alu.t1_invert = true;
        c.alu.carry_invert = true;
      },
      0b001 => { // NOT
        c.alu.mode = control::AluMode::Add;
        c.alu.t0_zero = true;
        c.alu.t1_invert = true;
      },
      0b100 => { // SL
        c.alu.mode = control::AluMode::Shift;
        c.alu.extend = false;
        c.alu.direction = false;
      },
      0b110 => { // LSR
        c.alu.mode = control::AluMode::Shift;
        c.alu.extend = false;
        c.alu.direction = true;
      },
      0b111 => { // ASR
        c.alu.mode = control::AluMode::Shift;
        c.alu.extend = true;
        c.alu.direction = true;
      },
      value => return Err(Error::InvalidUnaryOp(op, value)),
    }
    Ok(())
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Microcode {
  address: AddressSelect,

  data: [(DataSelect, Direction); 2],

  alu_mode: AluMode,
  set_flags: bool,

  pc_increment: bool,

  s_count: Option<Direction>,

  interrupt: bool,
  halt: bool,
}

impl Microcode {
  fn new() -> Microcode {
    Microcode {
      address: AddressSelect::A,

      data: [
        (DataSelect::None, Direction::Const),
        (DataSelect::None, Direction::Const),
      ],

      alu_mode: AluMode::None,
      set_flags: false,

      pc_increment: false,

      s_count: None,

      interrupt: false,
      halt: false,
    }
  }

  pub fn decode(&self, op: u16, branch: Option<u16>) -> Result<Control> {
    let mut c = Control::new();

    self.address.decode(op, &mut c);

    self.data[0].0.decode(op, self.data[0].1.parse(op, false), &mut c)?;
    self.data[1].0.decode(op, self.data[1].1.parse(op, true), &mut c)?;

    self.alu_mode.decode(op, &mut c)?;
    c.alu.set_flags = self.set_flags;

    if let Some(mask) = branch {
      c.branch.negate = (op & 0x0800) != 0;
      c.branch.condition = match op & 0x0007 {
        0 => control::Condition::Always,
        2 => control::Condition::Zero,
        3 => control::Condition::Sign,
        4 => control::Condition::Carry,
        5 => control::Condition::CarryNotZero,
        6 => control::Condition::Overflow,
        7 => control::Condition::OverflowNotZero,
        value => return Err(Error::InvalidCondition(op, value)),
      };

      c.link = (op & mask) != 0;
      if c.link && self.pc_increment {
        c.lr.increment = true;
      }
    }

    if self.pc_increment && !c.pc.load {
      c.pc.increment = true;
    }

    if let Some(direction) = self.s_count {
      c.s[((op & 0x0200) != 0) as usize].count = true;
      c.s[((op & 0x0200) != 0) as usize].direction = direction.parse(op, true);
    }

    c.interrupt = self.interrupt;

    if self.halt {
      c.halt = (op & 0x0080) != 0;
    }

    Ok(c)
  }
}


pub type MicrocodeArray = [Microcode; 46];
pub fn array() -> MicrocodeArray {
  [
    { // 0 FETCH
      let mut m = Microcode::new();
      m.address = AddressSelect::ProgramCounter;
      m.data = [
        (DataSelect::Memory, Direction::Const),
        (DataSelect::I, Direction::Const),
      ];
      m.pc_increment = true;
      m
    },
    { // 1 LD r,r
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterOne, Direction::Const),
        (DataSelect::RegisterZero, Direction::Const),
      ];
      m
    },
    { // 2 LD r,(r) / ALU r,(r) / JMl (r)
      let mut m = Microcode::new();
      m.address = AddressSelect::A;
      m.data = [
        (DataSelect::RegisterOne, Direction::Const),
        (DataSelect::A, Direction::Const),
      ];
      m
    },
    { // 3 LD r,(r) / LD r,(word) / LD r,(r+r)
      let mut m = Microcode::new();
      m.address = AddressSelect::A;
      m.data = [
        (DataSelect::Memory, Direction::Near),
        (DataSelect::RegisterZero, Direction::Near),
      ];
      m
    },
    { // 4 LD r,word
      let mut m = Microcode::new();
      m.address = AddressSelect::ProgramCounter;
      m.data = [
        (DataSelect::Memory, Direction::Near),
        (DataSelect::RegisterZero, Direction::Near),
      ];
      m.pc_increment = true;
      m
    },
    { // 5 LD r,(word) / ALU r,(word) / JMl (word) / LD x,(word)
      let mut m = Microcode::new();
      m.address = AddressSelect::ProgramCounter;
      m.data = [
        (DataSelect::Memory, Direction::Const),
        (DataSelect::A, Direction::Const),
      ];
      m.pc_increment = true;
      m
    },
    { // 6 LD r,(r+r) / ALU r,(r+r) / JMl (r+r)
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterOne, Direction::Const),
        (DataSelect::T(false), Direction::Const),
      ];
      m
    },
    { // 7 LD r,(r+r) / ALU r,(r+r) / JMl (r+r) / LD x,(r+r)
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterTwo, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 8 LD r,(r+r) / ALU r,(r+r) / JMl (r+r) / LD x,(r+r)
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Add;
      m.data = [
        (DataSelect::Alu, Direction::Const),
        (DataSelect::A, Direction::Const),
      ];
      m
    },
    { // 9 LD r,b
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::SignedByte, Direction::Const),
        (DataSelect::RegisterZero, Direction::Const),
      ];
      m
    },
    { // 10 LD r,(u) / ALU r,(u) / JMl (u)
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::UnsignedByte, Direction::Const),
        (DataSelect::A, Direction::Const),
      ];
      m
    },
    { // 11 LD r,(u)
      let mut m = Microcode::new();
      m.address = AddressSelect::A;
      m.data = [
        (DataSelect::Memory, Direction::Far),
        (DataSelect::RegisterZero, Direction::Far),
      ];
      m
    },
    { // 12 ALU r,r / ALU r,(r) / ALU r,word / ALU r,(word) / ALU r,(r+r) / SET r,b,v / TEST r,b / UOP r
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Binary;
      m.data = [
        (DataSelect::RegisterZero, Direction::Const),
        (DataSelect::T(false), Direction::Const),
      ];
      m
    },
    { // 13 ALU r,r
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Binary;
      m.data = [
        (DataSelect::RegisterOne, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 14 ALU r,r / ALU r,(r) / ALU r,word / ALU r,(word) / ALU r,(r+r)
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Binary;
      m.data = [
        (DataSelect::Alu, Direction::Const),
        (DataSelect::RegisterZero, Direction::Const),
      ];
      m.set_flags = true;
      m
    },
    { // 15 ALU r,(r) / ALU r,(word) / ALU r,(r+r)
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Binary;
      m.address = AddressSelect::A;
      m.data = [
        (DataSelect::Memory, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 16 ALU r,word
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Binary;
      m.address = AddressSelect::ProgramCounter;
      m.data = [
        (DataSelect::Memory, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m.pc_increment = true;
      m
    },
    { // 17 ALU r,b, ALU r,(u)
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Short;
      m.data = [
        (DataSelect::RegisterZero, Direction::Const),
        (DataSelect::T(false), Direction::Const),
      ];
      m
    },
    { // 18 ALU r,b
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Short;
      m.data = [
        (DataSelect::SignedByte, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 19 ALU r,b / ALU r,(u)
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Short;
      m.data = [
        (DataSelect::Alu, Direction::Const),
        (DataSelect::RegisterZero, Direction::Const),
      ];
      m.set_flags = true;
      m
    },
    { // 20 ALU r,(u)
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Short;
      m.address = AddressSelect::A;
      m.data = [
        (DataSelect::Memory, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 21 JMl r
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterOne, Direction::Const),
        (DataSelect::ProgramCounter, Direction::Const),
      ];
      m
    },
    { // 22 JMl (r) / JMl (word) / JMl (r+r)
      let mut m = Microcode::new();
      m.address = AddressSelect::A;
      m.data = [
        (DataSelect::Memory, Direction::Const),
        (DataSelect::ProgramCounter, Direction::Const),
      ];
      m
    },
    { // 23 JMl word
      let mut m = Microcode::new();
      m.address = AddressSelect::ProgramCounter;
      m.data = [
        (DataSelect::Memory, Direction::Const),
        (DataSelect::ProgramCounter, Direction::Const),
      ];
      m.pc_increment = true;
      m
    },
    { // 24 JMl b
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::ProgramCounter, Direction::Const),
        (DataSelect::T(false), Direction::Const),
      ];
      m
    },
    { // 25 JMl b
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::SignedByte, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 26 JMl b
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Add;
      m.data = [
        (DataSelect::Alu, Direction::Const),
        (DataSelect::ProgramCounter, Direction::Const),
      ];
      m
    },
    { // 27 RET
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::LinkRegister, Direction::Const),
        (DataSelect::ProgramCounter, Direction::Const),
      ];
      m
    },
    { // 28 RETs
      let mut m = Microcode::new();
      m.address = AddressSelect::S;
      m.data = [
        (DataSelect::Memory, Direction::Const),
        (DataSelect::ProgramCounter, Direction::Const),
      ];
      m.s_count = Some(Direction::Pop);
      m
    },
    { // 29 PUTs/POPs
      let mut m = Microcode::new();
      m.address = AddressSelect::S;
      m.data = [
        (DataSelect::Memory, Direction::Near),
        (DataSelect::None, Direction::Near),
      ];
      m.s_count = Some(Direction::Near);
      m
    },
    { // 30 SET F,b,v
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::F, Direction::Const),
        (DataSelect::T(false), Direction::Const),
      ];
      m
    },
    { // 31 SET F,b,v
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::Bitmask, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 32 SET F,b,v
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Set;
      m.data = [
        (DataSelect::Alu, Direction::Const),
        (DataSelect::F, Direction::Const),
      ];
      m
    },
    { // 33 SET r,b,v
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Set;
      m.data = [
        (DataSelect::Alu, Direction::Const),
        (DataSelect::RegisterZero, Direction::Const),
      ];
      m
    },
    { // 34 TEST r,b
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Test;
      m.data = [
        (DataSelect::Bitmask, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m.set_flags = true;
      m
    },
    { // 35 UOP r
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Unary;
      m.data = [
        (DataSelect::Alu, Direction::Const),
        (DataSelect::RegisterZero, Direction::Const),
      ];
      m.set_flags = true;
      m
    },
    { // 36 LD x,r
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::X, Direction::Near),
        (DataSelect::RegisterZero, Direction::Near),
      ];
      m
    },
    { // 37 LD x,(r)
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterZero, Direction::Const),
        (DataSelect::A, Direction::Const),
      ];
      m
    },
    { // 38 LD x,(r) / LD x,(word) / LD x,(r+r)
      let mut m = Microcode::new();
      m.address = AddressSelect::A;
      m.data = [
        (DataSelect::Memory, Direction::Near),
        (DataSelect::X, Direction::Near),
      ];
      m
    },
    { // 39 LD x,word
      let mut m = Microcode::new();
      m.address = AddressSelect::ProgramCounter;
      m.data = [
        (DataSelect::Memory, Direction::Near),
        (DataSelect::X, Direction::Near),
      ];
      m.pc_increment = true;
      m
    },
    { // 40 LD x,(r+r)
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterZero, Direction::Const),
        (DataSelect::T(false), Direction::Const),
      ];
      m
    },
    { // 41 INT
      let mut m = Microcode::new();
      m.address = AddressSelect::S;
      m.data = [
        (DataSelect::ProgramCounter, Direction::Const),
        (DataSelect::Memory, Direction::Const),
      ];
      m.s_count = Some(Direction::Put);
      m
    },
    { // 42 INT
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::Interrupt, Direction::Const),
        (DataSelect::ProgramCounter, Direction::Const),
      ];
      m.halt = true;
      m
    },
    { // 43 NOP
      let mut m = Microcode::new();
      m.halt = true;
      m
    },
    { // 44 INIT
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::Startup, Direction::Const),
        (DataSelect::A, Direction::Const),
      ];
      m
    },
    { // 45 INIT
      let mut m = Microcode::new();
      m.address = AddressSelect::A;
      m.data = [
        (DataSelect::Memory, Direction::Const),
        (DataSelect::ProgramCounter, Direction::Const),
      ];
      m
    },
  ]
}
