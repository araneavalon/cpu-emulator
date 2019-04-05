
use crate::control::control::{
  self,
  Control,
};


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum AddressSelect {
  A,
  ProgramCounter,
  S,
}

impl AddressSelect {
  fn decode(&self, op: u16, c: &mut Control) {
    match self {
      AddressSelect::S => {
        if (op & 0x0040) == 0 {
          c.address = control::Address::StackZero;
        } else {
          c.address = control::Address::StackOne;
        }
      },
      AddressSelect::A => {
        c.address = control::Address::A;
      },
      AddressSelect::ProgramCounter => {
        c.address = control::Address::ProgramCounter;
      },
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DataSelect {
  None,

  // Load/Out
  RegisterZero,
  X, // PC, LR, S0, S1
  S,
  ProgramCounter,
  LinkRegister,
  F,
  MemoryWord,
  MemoryW,

  // Out
  RegisterOne,
  RegisterTwo,
  Alu,
  SignedByte,
  EByte,
  WordOffset,
  Bitmask,
  Interrupt,

  // Load
  T(bool),
  A,
  I,
}

macro_rules! bi {
  ( $c:expr, $d:expr, $v:expr ) => {
    { if $d { $c.load = $v; } else { $c.out = $v; } }
  };
}

impl DataSelect {
  fn decode(&self, op: u16, d: bool, c: &mut Control) {
    match self {
      DataSelect::None => (),

      DataSelect::RegisterZero => {
        bi!(c.register, d, DataSelect::parse_register(op, 0));
      },
      DataSelect::RegisterOne => {
        assert!(!d, "Can not load via register offset 3.");
        c.register.out = DataSelect::parse_register(op, 3);
      },
      DataSelect::RegisterTwo => {
        assert!(!d, "Can not load via register offset 6.");
        c.register.out = DataSelect::parse_register(op, 6);
      },

      DataSelect::T(s) => {
        assert!(d, "Can not read from alu T registers.");
        c.alu.t[*s as usize].load = true;
      },
      DataSelect::Alu => {
        assert!(!d, "Can not load to alu output.");
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
      DataSelect::S => {
        bi!(c.s[((op & 0x0040) != 0) as usize], d, true);
      },
      DataSelect::X => {
        match (op & 0x0018) >> 3 {
          0x0 => bi!(c.pc, d, true),
          0x1 => bi!(c.lr, d, true),
          0x2 => bi!(c.s[0], d, true),
          0x3 => bi!(c.s[1], d, true),
          _ => panic!("Invalid value for X."),
        }
      },

      DataSelect::A => {
        assert!(!d, "Can not read from address register.");
        c.a.load = true;
      },
      DataSelect::MemoryWord => {
        bi!(c.memory, d, true);
        c.memory.word = true;
      },
      DataSelect::MemoryW => {
        bi!(c.memory, d, true);
        c.memory.word = (op & 0x0200) == 0;
      },

      DataSelect::I => {
        assert!(!d, "Can not read from I register.");
        c.i.load = true;
      },
      DataSelect::SignedByte => {
        assert!(d, "Can not write to I(sb) register.");
        c.i.mode = control::IMode::SignedByte;
      },
      DataSelect::EByte => {
        assert!(d, "Can not write to I(eb) register.");
        if (op & 0x0800) != 0 {
          c.i.mode = control::IMode::SignedByte;
        } else {
          c.i.mode = control::IMode::UnsignedByte;
        }
      },
      DataSelect::WordOffset => {
        assert!(d, "Can not write to I(wo) register.");
        c.i.mode = control::IMode::WordOffset;
      },
      DataSelect::Bitmask => {
        assert!(d, "Can not write to I(bm) register.");
        c.i.mode = control::IMode::Bitmask;
      },

      DataSelect::Interrupt => {
        assert!(d, "Can not write to I(int) register.");
        c.i.mode = control::IMode::Interrupt;
      },
    }
  }

  fn parse_register(op: u16, offset: u16) -> control::Register {
    match (op >> offset) & 0x0007 {
      0x0 => control::Register::Zero,
      0x1 => control::Register::One,
      0x2 => control::Register::Two,
      0x3 => control::Register::Three,
      0x4 => control::Register::Four,
      0x5 => control::Register::Five,
      0x6 => control::Register::Six,
      0x7 => control::Register::Seven,
      _ => panic!("Invalid value for register."),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
  Const,
  Zero,
}

impl Direction {
  fn parse(&self, op: u16, default: bool) -> bool {
    match self {
      Direction::Const => default,
      Direction::Zero => default ^ ((op & 0x0400) != 0),
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
  fn decode(&self, op: u16, c: &mut Control) {
    match self {
      AluMode::None => (),

      AluMode::Unary => AluMode::decode_unary((op & 0x3100) >> 11, c),
      AluMode::Short => AluMode::decode_binary((op & 0x3000) >> 11, c),
      AluMode::Binary => AluMode::decode_binary((op & 0x3100) >> 11, c),

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
  }

  fn decode_binary(op: u16, c: &mut Control) {
    match op {
      0x0 => c.alu.mode = control::AluMode::Add, // ADD
      0x1 => c.alu.mode = control::AluMode::And, // AND
      0x2 => {
        c.alu.mode = control::AluMode::Add; // SUB
        c.alu.t1_invert = true;
        c.alu.carry_invert = true;
      },
      0x3 => {
        c.alu.mode = control::AluMode::Add; // SBN
        c.alu.t1_invert = true;
        c.alu.carry_invert = true;
        if c.alu.t[0].load | c.alu.t[1].load {
          c.alu.t[0].load = !c.alu.t[0].load;
          c.alu.t[1].load = !c.alu.t[1].load;
        }
      },
      0x4 => {
        c.alu.mode = control::AluMode::Add; // CMP
        c.alu.t1_invert = true;
        c.alu.carry_invert = true;
        c.alu.out = false;
      }
      0x5 => c.alu.mode = control::AluMode::Or, // OR
      0x6 => {
        c.alu.mode = control::AluMode::Add; // CPN
        c.alu.t1_invert = true;
        c.alu.carry_invert = true;
        if c.alu.t[0].load | c.alu.t[1].load {
          c.alu.t[0].load = !c.alu.t[0].load;
          c.alu.t[1].load = !c.alu.t[1].load;
        }
        c.alu.out = false;
      },
      0x7 => c.alu.mode = control::AluMode::Xor, // XOR
      _ => panic!("Invalid value for binary op."),
    }
  }

  fn decode_unary(op: u16, c: &mut Control) {
    if op == 0b000 {
      c.alu.mode = control::AluMode::Add; // NEG
      c.alu.t0_zero = true;
      c.alu.t1_invert = true;
      c.alu.carry_invert = true;
    } else if op == 0b001 {
      c.alu.mode = control::AluMode::Add; // NOT
      c.alu.t0_zero = true;
      c.alu.t1_invert = true;
    } else {
      c.alu.mode = control::AluMode::Shift;
      c.alu.direction = (op & 0b100) == 0;
      c.alu.word = (op & 0b001) == 0;
      c.alu.extend = (op & 0b010) != 0; // Ignored for L-shifts.
    }
  }
}


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Microcode {
  address: AddressSelect,

  data: [(DataSelect, Direction); 2],

  alu_mode: AluMode,
  set_flags: bool,

  branch: bool,
  link: bool,
  pc_increment: bool,

  s_count: bool,
  stack_sequence: bool,

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

      branch: false,
      link: false,
      pc_increment: false,

      s_count: false,
      stack_sequence: false,

      interrupt: false,
      halt: false,
    }
  }

  pub fn decode(&self, op: u16) -> Control {
    let mut c = Control::new();

    self.address.decode(op, &mut c);

    self.data[0].0.decode(op, self.data[0].1.parse(op, false), &mut c);
    self.data[1].0.decode(op, self.data[1].1.parse(op, true), &mut c);

    self.alu_mode.decode(op, &mut c);
    c.alu.set_flags = self.set_flags;

    if self.branch {
      c.branch.negate = (op & 0x0800) != 0;
      c.branch.condition = match op & 0x0007 {
        0x0 => control::Condition::Always,
        0x1 => control::Condition::Always,
        0x2 => control::Condition::Zero,
        0x3 => control::Condition::Sign,
        0x4 => control::Condition::Carry,
        0x5 => control::Condition::CarryNotZero,
        0x6 => control::Condition::Overflow,
        0x7 => control::Condition::OverflowNotZero,
        _ => panic!("Invalid value for condition."),
      };
    }

    if self.link & ((op & 0x1000) != 0) {
      c.link = true;
      if self.pc_increment {
        c.lr.increment = true;
      }
    }

    if self.pc_increment & !c.pc.load {
      c.pc.increment = true;
    }

    if self.s_count {
      c.s[((op & 0x0040) != 0) as usize].count = true;
      c.s[((op & 0x0040) != 0) as usize].direction = Direction::Zero.parse(op, false);
    }

    c.stack_sequence = self.stack_sequence;

    c.interrupt = self.interrupt;

    if self.halt {
      c.halt = (op & 0x0200) != 0;
    }

    c
  }
}


pub fn array() -> [Microcode; 46] {
  [
    { // 0
      let mut m = Microcode::new();
      m.address = AddressSelect::ProgramCounter;
      m.data = [
        (DataSelect::MemoryWord, Direction::Const),
        (DataSelect::I, Direction::Const),
      ];
      m.pc_increment = true;
      m
    },
    { // 1
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Short;
      m.data = [
        (DataSelect::RegisterZero, Direction::Const),
        (DataSelect::T(false), Direction::Const),
      ];
      m
    },
    { // 2
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Short;
      m.data = [
        (DataSelect::EByte, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 3
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Short;
      m.set_flags = true;
      m.data = [
        (DataSelect::Alu, Direction::Const),
        (DataSelect::RegisterZero, Direction::Const),
      ];
      m
    },
    { // 4
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterOne, Direction::Const),
        (DataSelect::T(false), Direction::Const),
      ];
      m
    },
    { // 5
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterTwo, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 6
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Add;
      m.data = [
        (DataSelect::Alu, Direction::Const),
        (DataSelect::A, Direction::Const),
      ];
      m
    },
    { // 7
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Binary;
      m.data = [
        (DataSelect::RegisterZero, Direction::Const),
        (DataSelect::T(false), Direction::Const),
      ];
      m
    },
    { // 8
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Binary;
      m.address = AddressSelect::A;
      m.data = [
        (DataSelect::MemoryW, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 9
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Binary;
      m.set_flags = true;
      m.data = [
        (DataSelect::Alu, Direction::Const),
        (DataSelect::RegisterZero, Direction::Const),
      ];
      m
    },
    { // 10
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::S, Direction::Const),
        (DataSelect::T(false), Direction::Const),
      ];
      m
    },
    { // 11
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::WordOffset, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 12
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Binary;
      m.data = [
        (DataSelect::RegisterOne, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 13
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterOne, Direction::Const),
        (DataSelect::A, Direction::Const),
      ];
      m
    },
    { // 14
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Binary;
      m.address = AddressSelect::ProgramCounter;
      m.data = [
        (DataSelect::MemoryW, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m.pc_increment = true;
      m
    },
    { // 15
      let mut m = Microcode::new();
      m.address = AddressSelect::ProgramCounter;
      m.data = [
        (DataSelect::MemoryW, Direction::Const),
        (DataSelect::A, Direction::Const),
      ];
      m.pc_increment = true;
      m
    },
    { // 16
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterZero, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 17
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Unary;
      m.set_flags = true;
      m.data = [
        (DataSelect::Alu, Direction::Const),
        (DataSelect::RegisterOne, Direction::Const),
      ];
      m
    },
    { // 18
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::ProgramCounter, Direction::Const),
        (DataSelect::T(false), Direction::Const),
      ];
      m.branch = true;
      m.link = true;
      m
    },
    { // 19
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::SignedByte, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 20
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Add;
      m.data = [
        (DataSelect::Alu, Direction::Const),
        (DataSelect::ProgramCounter, Direction::Const),
      ];
      m
    },
    { // 21
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterOne, Direction::Const),
        (DataSelect::ProgramCounter, Direction::Const),
      ];
      m.branch = true;
      m.link = true;
      m
    },
    { // 22
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterOne, Direction::Const),
        (DataSelect::A, Direction::Const),
      ];
      m.branch = true;
      m.link = true;
      m
    },
    { // 23
      let mut m = Microcode::new();
      m.address = AddressSelect::A;
      m.data = [
        (DataSelect::MemoryWord, Direction::Zero),
        (DataSelect::ProgramCounter, Direction::Zero),
      ];
      m
    },
    { // 24
      let mut m = Microcode::new();
      m.address = AddressSelect::ProgramCounter;
      m.data = [
        (DataSelect::MemoryWord, Direction::Zero),
        (DataSelect::ProgramCounter, Direction::Zero),
      ];
      m.branch = true;
      m.link = true;
      m.pc_increment = true;
      m
    },
    { // 25
      let mut m = Microcode::new();
      m.address = AddressSelect::ProgramCounter;
      m.data = [
        (DataSelect::MemoryWord, Direction::Const),
        (DataSelect::A, Direction::Const),
      ];
      m.branch = true;
      m.link = true;
      m.pc_increment = true;
      m
    },
    { // 26
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::LinkRegister, Direction::Zero),
        (DataSelect::ProgramCounter, Direction::Zero),
      ];
      m.branch = true;
      m.link = true;
      m
    },
    { // 27
      let mut m = Microcode::new();
      m.address = AddressSelect::S;
      m.data = [
        (DataSelect::MemoryWord, Direction::Zero),
        (DataSelect::ProgramCounter, Direction::Zero),
      ];
      m.branch = true;
      m.link = true;
      m.s_count = true;
      m
    },
    { // 28
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterZero, Direction::Const),
        (DataSelect::T(false), Direction::Const),
      ];
      m
    },
    { // 29
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::Bitmask, Direction::Const),
        (DataSelect::T(true), Direction::Const),
      ];
      m
    },
    { // 30
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Set;
      m.set_flags = true;
      m.data = [
        (DataSelect::Alu, Direction::Const),
        (DataSelect::RegisterZero, Direction::Const),
      ];
      m
    },
    { // 31
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::F, Direction::Const),
        (DataSelect::T(false), Direction::Const),
      ];
      m
    },
    { // 32
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Set;
      m.data = [
        (DataSelect::Alu, Direction::Const),
        (DataSelect::F, Direction::Const),
      ];
      m
    },
    { // 33
      let mut m = Microcode::new();
      m.alu_mode = AluMode::Test;
      m.set_flags = true;
      m
    },
    { // 34
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::EByte, Direction::Const),
        (DataSelect::RegisterZero, Direction::Const),
      ];
      m
    },
    { // 35
      let mut m = Microcode::new();
      m.address = AddressSelect::A;
      m.data = [
        (DataSelect::MemoryW, Direction::Zero),
        (DataSelect::RegisterZero, Direction::Zero),
      ];
      m
    },
    { // 36
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterOne, Direction::Const),
        (DataSelect::RegisterZero, Direction::Const),
      ];
      m
    },
    { // 37
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterOne, Direction::Const),
        (DataSelect::A, Direction::Const),
      ];
      m
    },
    { // 38
      let mut m = Microcode::new();
      m.address = AddressSelect::ProgramCounter;
      m.data = [
        (DataSelect::MemoryW, Direction::Zero),
        (DataSelect::RegisterZero, Direction::Zero),
      ];
      m.pc_increment = true;
      m
    },
    { // 39
      let mut m = Microcode::new();
      m.address = AddressSelect::ProgramCounter;
      m.data = [
        (DataSelect::MemoryWord, Direction::Const),
        (DataSelect::A, Direction::Const),
      ];
      m.pc_increment = true;
      m
    },
    { // 40
      let mut m = Microcode::new();
      m.data = [
        (DataSelect::RegisterZero, Direction::Zero),
        (DataSelect::X, Direction::Zero),
      ];
      m
    },
    { // 41
      let mut m = Microcode::new();
      m.address = AddressSelect::A;
      m.data = [
        (DataSelect::MemoryWord, Direction::Zero),
        (DataSelect::X, Direction::Zero),
      ];
      m
    },
    { // 42
      let mut m = Microcode::new();
      m.address = AddressSelect::ProgramCounter;
      m.data = [
        (DataSelect::MemoryWord, Direction::Zero),
        (DataSelect::X, Direction::Zero),
      ];
      m.pc_increment = true;
      m
    },
    { // 43
      let mut m = Microcode::new();
      m.address = AddressSelect::S;
      m.data = [
        (DataSelect::MemoryWord, Direction::Zero),
        (DataSelect::None, Direction::Zero),
      ];
      m.stack_sequence = true;
      m.s_count = true;
      m
    },
    { // 44
      let mut m = Microcode::new();
      m.interrupt = true;
      m.halt = true;
      m.link = true;
      m.data = [
        (DataSelect::Interrupt, Direction::Const),
        (DataSelect::ProgramCounter, Direction::Const),
      ];
      m
    },
    { // 45
      let mut m = Microcode::new();
      m.halt = true;
      m
    },
  ]
}
