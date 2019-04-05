
use crate::control::{
  Control,
  AluMode,
};
use crate::components::flags::Flag;
use crate::components::BusComponent;


#[derive(Debug)]
pub struct Alu {
  control: Control,
  t: [u16; 2],
}

impl Alu {
  pub fn new() -> Alu {
    Alu {
      control: Control::new(),
      t: [0x0000; 2],
    }
  }

  fn flags(&self, zero: bool, sign: bool, carry: bool, overflow: bool) -> [bool; 8] {
    let mut flags = [false; 8];
    flags[Flag::Zero as usize] = zero;
    flags[Flag::Sign as usize] = sign;
    flags[Flag::Carry as usize] = carry;
    flags[Flag::Overflow as usize] = overflow;
    flags
  }

  fn shift(&self) -> (u16, [bool; 8]) {
    let (value, carry) = if self.control.alu.direction {
      if self.control.alu.word {
        (self.t[1] << 1, (self.t[1] & 0x8000) != 0)
      } else {
        let v = (self.t[1] as u8) << 1;
        ((0xFF00 * ((v >> 7) as u16)) | (v as u16), (self.t[1] & 0x0080) != 0)
      }
    } else {
      let value = match (self.control.alu.word, self.control.alu.extend) {
        (true, true)   => ((self.t[1] as i16) >> 1) as u16,
        (true, false)  => self.t[1] >> 1,
        (false, true)  => ((self.t[1] as i8) >> 1) as u16,
        (false, false) => ((self.t[1] as u8) >> 1) as u16,
      };
      (value, (self.t[1] & 0x0001) != 0)
    };
    (value, self.flags(value == 0, (value as i16) < 0, carry, false))
  }

  fn binary<F>(&self, func: F) -> (u16, [bool; 8])
  where F: Fn(i32, i32, i32) -> i32 {
    let t0 = (self.t[0] & if self.control.alu.t0_zero { 0x0000 } else { 0xFFFF }) as i32;
    let t1 = (self.t[1] ^ if self.control.alu.t1_invert { 0xFFFF } else { 0x0000 }) as i32;
    let c = self.control.alu.carry_invert as i32;

    let value = func(t0, t1, c);
    let carry = self.control.alu.carry_invert ^ (value > (u16::max_value() as i32));
    let overflow = (value > (i16::max_value() as i32)) | (value < (i16::min_value() as i32));

    (value as u16, self.flags(value == 0, value < 0, carry, overflow))
  }

  fn calculate(&self) -> (u16, [bool; 8]) {
    match self.control.alu.mode {
      AluMode::Shift => self.shift(),
      AluMode::Add   => self.binary(|t0, t1,  c| t0 + t1 + c),
      AluMode::And   => self.binary(|t0, t1, _c| t0 & t1),
      AluMode::Or    => self.binary(|t0, t1, _c| t0 | t1),
      AluMode::Xor   => self.binary(|t0, t1, _c| t0 ^ t1),
    }
  }

  pub fn get_flags(&self) -> [bool; 8] {
    self.calculate().1
  }
}

impl BusComponent for Alu {
  fn set_control(&mut self, control: Control) {
    self.control = control;
  }

  fn load(&mut self, value: u16) {
    for i in 0..2 {
      if self.control.alu.t[i].load {
        self.t[i] = value;
      }
    }
  }

  fn data(&self) -> Option<u16> {
    if self.control.alu.out {
      Some(self.calculate().0)
    } else {
      None
    }
  }
}
