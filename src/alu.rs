
use std::collections::HashMap;
use std::fmt;

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
  o_value: u8,
}

impl Alu {
  pub fn new() -> Alu {
    Alu {
      control: control::Alu::new(),
      flags: HashMap::new(),
      t_value: 0x00,
      o_value: 0x00,
    }
  }

  pub fn get_flags(&self) -> &Flags {
    &self.flags
  }

  fn input_t(&self, state: &bus::State) -> Result<u8, Error> {
    use crate::control::AluSelect as Select;
    match self.control.TempSelect {
      Select::Zero   => Ok(0x00),
      Select::One    => Ok(0xFF),
      Select::Value  => Ok(self.t_value),
      Select::Invert => Ok(!self.t_value),
    }
  }

  fn input_i(&self, state: &bus::State) -> Result<u8, Error> {
    use crate::control::AluInput as Select;
    match self.control.Input {
      Select::Zero => Ok(0x00),
      Select::Data => state.read_data(),
      Select::Addr => Ok(state.read_addr()? as u8),
    }
  }

  fn calculate_add(&self, state: &bus::State, carry_select: control::AluSelect) -> Result<(u8, Flags), Error> {
    let temp: u16 = self.input_t(state)? as u16;
    let input: u16 = self.input_i(state)? as u16;
    let carry: u16 = match carry_select {
      control::AluSelect::Zero   => 0,
      control::AluSelect::One    => 1,
      control::AluSelect::Value  => self.control.Flags[&Flag::C] as u16,
      control::AluSelect::Invert => (!self.control.Flags[&Flag::C]) as u16,
    };

    let result = temp + input + carry;
    let carry_out = if self.control.Input == control::AluInput::Zero {
      false
    } else if carry_select == control::AluSelect::One || carry_select == control::AluSelect::Invert {
      result & 0xFF00 == 0
    } else {
      result & 0xFF00 != 0
    };
    let result = result as u8;

    Ok((result, hash_map!{
      Flag::Z => result == 0,
      Flag::C => carry_out,
      Flag::V => overflow(temp as u8, input as u8, result),
      Flag::S => sign(result),
    }))
  }

  fn calculate_bitwise<F>(&self, state: &bus::State, func: F) -> Result<(u8, Flags), Error>
    where F: Fn(u8, u8) -> u8
  {
    let temp = self.input_t(state)?;
    let input = self.input_i(state)?;
    let result = func(temp, input);
    Ok((result, hash_map!{
      Flag::Z => result == 0,
      Flag::C => false,
      Flag::V => overflow(temp, input, result),
      Flag::S => sign(result),
    }))
  }

  fn calculate_rotate(&self, state: &bus::State, direction: control::AluRotateDirection, include_carry: bool) -> Result<(u8, Flags), Error> {
    let v = self.input_i(state)?;
    let (result, wrap, mask) = match direction {
      control::AluRotateDirection::Right => (v >> 1, v & 0x01 << 7, 0x80),
      control::AluRotateDirection::Left => (v << 1, v & 0x80 >> 7, 0x01),
    };
    let (result, carry) = if include_carry {
      (result | if self.control.Flags[&Flag::C] { mask } else { 0x00 }, wrap != 0)
    } else {
      (result | wrap, self.control.Flags[&Flag::C])
    };
    Ok((result, hash_map!{
      Flag::Z => result == 0,
      Flag::C => carry,
      Flag::V => false,
      Flag::S => sign(result),
    }))
  }

  fn calculate(&self, state: &bus::State) -> Result<(u8, Flags), Error> {
    use crate::control::AluOperation as Operation;
    #[allow(non_snake_case)]
    match self.control.Operation {
      Operation::Add { Carry } => self.calculate_add(state, Carry),
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
    Ok(())
  }

  fn read(&self) -> bus::State {
    bus::State {
      data: if let control::Write::Write = self.control.Data {
        Some(self.o_value)
      } else {
        None
      },
      addr: if let control::Write::Write = self.control.Addr {
        Some(bus::Addr::Low(self.o_value))
      } else {
        None
      },
    }
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    if let control::Write::Write = self.control.Output {
      let (value, flags) = self.calculate(state)?;
      self.o_value = value;
      self.flags = flags;
    }

    if let control::Read::Read = self.control.Temp {
      self.t_value = state.read_data()?;
    }

    Ok(())
  }
}

impl fmt::Display for Alu {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Alu(Temp={:#X} Output={:#X})", self.t_value, self.o_value)
  }
}

impl fmt::Debug for Alu {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // TODO FINISH.... ALL THIS BS
    write!(f, "Alu(Temp={:#X} Output={:#X})", self.t_value, self.o_value)
  }
}
