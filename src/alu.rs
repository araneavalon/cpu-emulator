
use std::collections::HashMap;
use std::fmt;

use crate::math::*;
use crate::bus;
use crate::control;
use crate::control::{Flags, Flag};
use crate::error::Error;


fn sign(v: u8) -> bool {
  (v & 0x80) != 0
}
fn overflow(a: u8, b: u8, r: u8) -> bool {
  (sign(a) == sign(b)) && (sign(a) != sign(r))
}

#[derive(PartialEq, Eq)]
pub struct Alu {
  control: control::Alu,
  flags: Flags,
  t_value: u8,
  o_value: [u8; 2],
}

impl Alu {
  pub fn new() -> Alu {
    Alu {
      control: control::Alu::new(),
      flags: HashMap::new(),
      t_value: 0x00,
      o_value: [0x00, 0x00],
    }
  }

  pub fn get_flags(&self) -> Flags {
    self.flags.clone()
  }

  fn input_t(&self) -> Result<u8, Error> {
    use crate::control::AluSelect as Select;
    match self.control.TempSelect {
      Select::Zero   => Ok(0x00),
      Select::One    => Ok(0xFF),
      Select::Value  => Ok(self.t_value),
      Select::Invert => Ok(!self.t_value),
    }
  }

  fn input_i(&self, state: &bus::State) -> Result<u16, Error> {
    use crate::control::AluInput as Select;
    match self.control.Input {
      Select::Zero => Ok(0x0000),
      Select::Data => Ok(state.read_data()? as u16),
      Select::Addr => Ok(state.read_addr()?),
    }
  }

  fn calculate_add(&self, state: &bus::State, signed: bool, carry_select: control::AluSelect) -> Result<([u8; 2], Flags), Error> {
    let temp: u32 = if signed {
      sign_extend(self.input_t()?) as u32
    } else {
      self.input_t()? as u32
    };
    let input: u32 = self.input_i(state)? as u32;
    let carry: u32 = match carry_select {
      control::AluSelect::Zero   => 0,
      control::AluSelect::One    => 1,
      control::AluSelect::Value  => self.control.Flags[&Flag::C] as u32,
      control::AluSelect::Invert => (!self.control.Flags[&Flag::C]) as u32,
    };

    let result = temp + input + carry;
    let carry_out = if self.control.Input == control::AluInput::Zero {
      false
    } else if (carry_select == control::AluSelect::One || carry_select == control::AluSelect::Invert) &&
        self.control.TempSelect != control::AluSelect::Zero {
      result & 0xFFFFFF00 == 0
    } else {
      result & 0xFFFFFF00 != 0
    };
    let result = to_bytes(result as u16);

    Ok((result, hash_map!{
      Flag::Z => result[0] == 0,
      Flag::C => carry_out,
      Flag::V => overflow(temp as u8, input as u8, result[0]),
      Flag::S => sign(result[0]),
    }))
  }

  fn calculate_bitwise<F>(&self, state: &bus::State, func: F) -> Result<([u8; 2], Flags), Error>
    where F: Fn(u8, u8) -> u8
  {
    let temp = self.input_t()?;
    let input = self.input_i(state)? as u8;
    let result = func(temp, input);
    Ok(([0x00, result], hash_map!{
      Flag::Z => result == 0,
      Flag::C => false,
      Flag::V => overflow(temp, input, result),
      Flag::S => sign(result),
    }))
  }

  fn calculate_rotate(&self, state: &bus::State, direction: control::AluRotateDirection, include_carry: bool) -> Result<([u8; 2], Flags), Error> {
    let v = self.input_i(state)? as u8;
    let (result, wrap, mask) = match direction {
      control::AluRotateDirection::Right => (v >> 1, v & 0x01 << 7, 0x80),
      control::AluRotateDirection::Left => (v << 1, v & 0x80 >> 7, 0x01),
    };
    let (result, carry) = if include_carry {
      (result | if self.control.Flags[&Flag::C] { mask } else { 0x00 }, wrap != 0)
    } else {
      (result | wrap, self.control.Flags[&Flag::C])
    };
    Ok(([0x00, result], hash_map!{
      Flag::Z => result == 0,
      Flag::C => carry,
      Flag::V => false,
      Flag::S => sign(result),
    }))
  }

  fn calculate(&self, state: &bus::State) -> Result<([u8; 2], Flags), Error> {
    use crate::control::AluOperation as Operation;
    #[allow(non_snake_case)]
    match self.control.Operation {
      Operation::Add { SignExtend, Carry } => self.calculate_add(state, SignExtend, Carry),
      Operation::And => self.calculate_bitwise(state, |a, b| a & b),
      Operation::Or  => self.calculate_bitwise(state, |a, b| a | b),
      Operation::Xor => self.calculate_bitwise(state, |a, b| a ^ b),
      Operation::Rotate { Direction, Carry } => self.calculate_rotate(state, Direction, Carry),
    }
  }
}

impl bus::Device<control::Alu> for Alu {
  fn update(&mut self, control: control::Alu) -> Result<(), Error> {
    self.control = control;
    self.flags = self.control.Flags.clone();
    Ok(())
  }

  fn read(&self) -> Result<bus::State, Error> {
    Ok(bus::State {
      data: if let control::Write::Write = self.control.Data {
        Some(self.o_value[1])
      } else {
        None
      },
      addr: if let control::Write::Write = self.control.Addr {
        Some(from_bytes(&self.o_value))
      } else {
        None
      },
    })
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    if let control::Read::Read = self.control.Temp {
      self.t_value = state.read_data()?;
    }

    let (value, flags) = self.calculate(state)?;
    self.flags = flags;
    if let control::Write::Write = self.control.Output {
      self.o_value = value;
    }

    Ok(())
  }
}

impl fmt::Display for Alu {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Alu(Temp={:#X} Output={:#X})", self.t_value, from_bytes(&self.o_value))
  }
}

impl fmt::Debug for Alu {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // TODO FINISH.... ALL THIS BS
    write!(f, "Alu(Temp={:#X} Output={:#X})", self.t_value, from_bytes(&self.o_value))
  }
}
