
use crate::control::{
  Control,
  Register,
};


macro_rules! bi {
  ( $c:expr, $d:expr, $v:expr ) => {
    { if $d { $c.load = $v; } else { $c.out = $v; } }
  };
}

#[derive(Debug)]
pub struct StackSequence {
  cycle: u16,
  direction: bool,
  registers: u16,
  base: Control,
}

impl StackSequence {
  pub fn new(op: u16, base: Control) -> StackSequence {
    StackSequence {
      cycle: 0,
      direction: (op & 0x400) != 0,
      registers: ((op & 0x0800) >> 2) | ((op & 0x0380) >> 1) | (op & 0x003f),
      base: base,
    }
  }
}

impl Iterator for StackSequence {
  type Item = Control;

  fn next(&mut self) -> Option<Control> {
    while self.cycle <= 9 {
      let bit = if self.direction { 9 - self.cycle } else { self.cycle };
      if ((self.registers >> bit) & 1) != 0 {
        let mut c = self.base.clone();
        match bit {
          0 => bi!(c.register, self.direction, Register::Zero),
          1 => bi!(c.register, self.direction, Register::One),
          2 => bi!(c.register, self.direction, Register::Two),
          3 => bi!(c.register, self.direction, Register::Three),
          4 => bi!(c.register, self.direction, Register::Four),
          5 => bi!(c.register, self.direction, Register::Five),
          6 => bi!(c.register, self.direction, Register::Six),
          7 => bi!(c.register, self.direction, Register::Seven),
          8 => bi!(c.flags, self.direction, true),
          9 => { if self.direction { c.pc.load = true; } else { c.lr.out = true; } },
          _ => panic!("It should be impossible to get this value."),
        }
        return Some(c)
      }
    }
    None
  }
}
