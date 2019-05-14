
use std::fmt;
use crate::control::{
  Control,
  AluMode,
};
use crate::error::Result;
use super::flags::Flag;
use super::BusComponent;


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
      (self.t[1] << 1, (self.t[1] & 0x8000) != 0)
    } else if self.control.alu.extend {
      (((self.t[1] as i16) >> 1) as u16, (self.t[1] & 0x0001) != 0)
    } else {
      (self.t[1] >> 1, (self.t[1] & 0x0001) != 0)
    };
    (value, self.flags(value == 0, (value as i16) < 0, carry, false))
  }

  fn binary<F>(&self, func: F) -> (u16, [bool; 8])
  where F: Fn(i32, i32, i32) -> i32 {
    let invert = self.control.alu.carry_invert;

    let t0 = (self.t[0] & if self.control.alu.t0_zero { 0x0000 } else { 0xFFFF }) as i32;
    let t1 = (self.t[1] ^ if self.control.alu.t1_invert { 0xFFFF } else { 0x0000 }) as i32;
    let c = invert as i32;

    let value = func(t0, t1, c);
    let carry = invert ^ (value > (u16::max_value() as i32));
    let overflow = invert ^ (value > (i16::max_value() as i32)) | (value < (i16::min_value() as i32));

    (value as u16, self.flags((value & 0x0000FFFF) == 0, (value & 0x00008000) != 0, carry, overflow))
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

  fn fmt_t(&self, f: &mut fmt::Formatter, t: usize) -> fmt::Result {
    let v = self.t[t];
    if self.control.alu.t[t].load {
      write!(f, " T{}  <= 0x{:04X}  {:5}  {:5}", t, v, v, v as i16)?;
    } else {
      write!(f, " T{}  == 0x{:04X}  {:5}  {:5}", t, v, v, v as i16)?;
    }
    Ok(())
  }
}

impl fmt::Display for Alu {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let (value, flags_arr) = self.calculate();
    let mut flags = 0x0000;
    for i in 0..8 {
      flags |= (flags_arr[i] as u16) << i;
    }
    // TODO include information about alu settings?

    self.fmt_t(f, 0)?;
    write!(f, "\n")?;
    self.fmt_t(f, 1)?;
    write!(f, "\n")?;
    if self.control.alu.out {
      write!(f, " ALU => 0x{:04X}  {:5}  {:5}\n        0b{:016b}", value, value, value as i16, flags)?;
    } else {
      write!(f, " ALU == 0x{:04X}  {:5}  {:5}\n        0b{:016b}", value, value, value as i16, flags)?;
    }
    Ok(())
  }
}

impl BusComponent for Alu {
  fn name(&self) -> &'static str {
    "Alu"
  }

  fn set_control(&mut self, control: Control) {
    self.control = control;
  }

  fn load(&mut self, value: u16) -> Result<()> {
    for i in 0..2 {
      if self.control.alu.t[i].load {
        self.t[i] = value;
      }
    }
    Ok(())
  }

  fn data(&self) -> Result<Option<u16>> {
    if self.control.alu.out {
      Ok(Some(self.calculate().0))
    } else {
      Ok(None)
    }
  }
}
