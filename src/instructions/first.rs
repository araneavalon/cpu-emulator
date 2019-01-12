
use std::collections::HashMap;

use crate::instructions;
use crate::instructions::Micro;
use crate::control::*;


enum Register { A, B, X, Y };
enum Address {
  Address,
  IndexedAddress(Register),
  IndirectAddress,
  IndirectIndexedAddress(Register),
};

mod register {
  fn set(control: &mut Control, register: Register, value: ReadWrite) {
    match register {
      Register::A => c.A.Data = value,
      Register::B => c.B.Data = value,
      Register::X => c.X.Data = value,
      Register::Y => c.Y.Data = value,
    }
  }
}

mod memory {
  fn addr() -> Vec<Micro> {
    vec![
      Micro::Static({
        let mut c = Control::new();
        c.ProgramCounter.Addr = ReadWrite::Write;
        c.Memory.Data = ReadWrite::Write;
        c.H.Data = ReadWrite::Read;
        c.H.Latch = Write::Write;
        c
      }),
      Micro::Static({
        let mut c = Control::new();
        c.ProgramCounter.Count = ProgramCounterCount::Increment;
        c.ProgramCounter.Addr =  ReadWrite::Write;
        c.Memory.Data = ReadWrite::Write;
        c.L.Data = ReadWrite::Read;
        c
      }),
      Micro::Static({
        let mut c = Control::new();
        c.ProgramCounter.Count = ProgramCounterCount::Increment;
        c.H.Addr = Write::Write;
        c.L.Addr = Write::Write;
        c
      })
    ]
  }
  fn idx_addr(index: Register) -> Vec<Control> {
    vec![
      Micro::Static({
        let mut c = Control::new();
        c.ProgramCounter.Addr = ReadWrite::Write;
        c.Memory.Data = ReadWrite::Write;
        c.H.Data = ReadWrite::Read;
        c.H.Latch = Write::Write;
        c
      }),
      Micro::Static({
        let mut c = Control::new();
        c.ProgramCounter.Count = ProgramCounterCount::Increment;
        c.ProgramCounter.Addr = ReadWrite::Write;
        c.Memory.Data = ReadWrite::Write;
        c.Alu.Temp.Data = Read::Read;
        c
      }),
      Micro::Static({
        let mut c = Control::new();
        c.ProgramCounter.Count = ProgramCounterCount::Increment;
        alu::add(&c);
        register::set(&c, index, ReadWrite::Write);
        c.Alu.Input = AluInput::Data;
        c.Alu.Output = Write::Write;
        c
      }),
      Micro::Flag(Flag::C, {
        let mut c = Control::new();
        c.H.Count = IncDec::Increment;
        c.Alu.Addr = Write::Write;
        c.H.Addr = Write::Write;
        c
      }, {
        let mut c = Control::new();
        c.Alu.Addr = Write::Write;
        c.H.Addr = Write::Write;
        c
      }),
    ]
  }
  fn ind_addr() -> Vec<Control> {

  }
  fn ind_idx_addr(index: Register) -> Vec<Control> {

  }
}


fn nop() -> Vec<Micro> {
  vec![Micro::Static(Control::new())]
}

fn set_flag(flag: Flag, value: bool) -> Vec<Micro> {
  let mut c = Control::new();
  match flag {
    Flag::C => c.FlagsRegister.C = Some(value),
    Flag::I => c.FlagsRegister.I = Some(value),
    _ => panic!("Can not call set_flag on {:?}.", flag),
  }
  vec![Micro::Static(c)]
}

fn push(src: Register) -> Vec<Micro> {
  vec![
    Micro::Static({
      let mut c = Control::new();
      c.StackPointer.Addr = Write::Write;
      c.Memory.Data = ReadWrite::Write;
      register::set(&c, dest, ReadWrite::Read);
      c
    }),
    Micro::Static({
      let mut c = Control::new();
      c.StackPointer.Count = IncDec::Increment;
      c
    }),
  ]
}
fn pop(dest: Register) -> Vec<Micro> {
  vec![
    Micro::Static({
      let mut c = Control::new();
      c.StackPointer.Addr = Write::Write;
      register::set(&c, dest, ReadWrite::Write);
      c.Memory.Data = ReadWrite::Read;
      c
    }),
    Micro::Static({
      let mut c = Control::new();
      c.StackPointer.Count = IncDec::Decrement;
      c
    }),
  ]
}

fn ld_reg_byte(dest: Register) -> Vec<Micro> {
  vec![
    Micro::Static({
      let mut c = Control::new();
      register::set(&c, dest, ReadWrite::Read);
      c.ProgramCounter.Addr = ReadWrite::Write;
      c.Memory.Data = ReadWrite::Write;
      c
    },
    Micro::Static({
      let mut c = Control::new();
      c.ProgramCounter.Count = ProgramCounterCount::Increment;
      c
    }),
  ]
}

fn ld_reg_reg(dest: Register, src: Register) -> Vec<Micro> {
  if dest == src {
    panic!("ld_reg_reg can not have identical dest and src arguments. Use ld_reg_byte instead.");
  }
  let mut c = Control::new();
  register::set(&c, dest, ReadWrite::Read);
  register::set(&c, src, ReadWrite::Write);
  vec![Micro::Static(c)]
}

fn ld_reg_mem_(register: Register, memory: Memory, r_value: ReadWrite, m_value: ReadWrite) -> Vec<Micro> {
  match memory {
    Memory::Address => {
      let mut m = memory::addr();
      if let Micro::Static(c) = &mut m[&2] {
        c.Memory.Data = m_value;
        register::set(c, register, r_value);
      }
      m
    }
    Memory::IndexedAddress(index) => {
      let mut m = memory::idx_addr(index);
      if let Micro::Flag(_, t, f) = &mut m[&3] {
        t.Memory.Data = m_value;
        register::set(t, register, r_value);
        f.Memory.Data = m_value;
        register::set(f, register, r_value);
      }
      m
    },
    Memory::IndirectAddress => (),
    Memory::IndirectIndexedAddress(index) => (),
  }
}
fn ld_reg_mem(dest: Register, src: Memory) -> Vec<Micro> {
  ld_reg_mem_(dest, src, ReadWrite::Read, ReadWrite::Write)
}
fn ld_mem_reg(src: Register, dest: Memory) -> Vec<Micro> {
  ld_reg_mem_(src, dest, ReadWrite::Write, ReadWrite::Read)
}


pub struct First {
  instructions: HashMap<u8, Vec<Micro>>;
}

impl First {
  pub fn new() -> First {
    let mut instructions = hash_map!{
      0b00000000 => instructions.insert(byte, nop()),
      0b00000001 => instructions.insert(byte, nop()), // TODO HLT
      0b00000010 => instructions.insert(byte, nop()), // TODO BRK
      0b00000011 => instructions.insert(byte, nop()), // UNDEFINED
      0b00000100 => instructions.insert(byte, set_flag(Flag::C, false)),
      0b00000101 => instructions.insert(byte, set_flag(Flag::C, true)),
      0b00000110 => instructions.insert(byte, set_flag(Flag::I, false)),
      0b00000111 => instructions.insert(byte, set_flag(Flag::I, true)),
      0b00001000 => // CALL addr
      0b00001001 => // CALL (addr)
      0b00001010 => // CALL (addr)+X
      0b00001011 => // CALL (addr)+Y
      0b00001100 => // RET
      0b00001101 => // RETI
      0b00001110 => // JMP addr
      0b00001111 => // JMP +byte
      0b00010000 => // JMP NZ,+byte
      0b00010001 => // JMP  Z,+byte
      0b00010010 => // JMP NC,+byte
      0b00010011 => // JMP  C,+byte
      0b00010100 => // JMP NV,+byte
      0b00010101 => // JMP  V,+byte
      0b00010110 => // JMP  P,+byte
      0b00010111 => // JMP  N,+byte
      0b00011000 => // INC X
      0b00011001 => // DEC X
      0b00011010 => // ADD X,byte
      0b00011011 => // SUB X,byte
      0b00011100 => // INC Y
      0b00011101 => // DEC Y
      0b00011110 => // ADD Y,byte
      0b00011111 => // SUB Y,byte
      0b00100000 => // CMP X,A
      0b00100001 => // CMP X,B
      0b00100010 => // CMP X,Y
      0b00100011 => // CMP X,byte
      0b00100100 => // CMP X,addr
      0b00100101 => // CMP X,addr+Y
      0b00100110 => // CMP X,(addr)
      0b00100111 => // CMP X,(addr)+Y
      0b00101000 => // CMP Y,A
      0b00101001 => // CMP Y,B
      0b00101010 => // CMP Y,X
      0b00101011 => // CMP Y,byte
      0b00101100 => // CMP Y,addr
      0b00101101 => // CMP Y,addr+X
      0b00101110 => // CMP Y,(addr)
      0b00101111 => // CMP Y,(addr)+X
      0b00110000 => // NOT A
      0b00110001 => // NEG A
      0b00110010 => // INC A
      0b00110011 => // DEC A
      0b00110100 => // RR  A
      0b00110101 => // RRC A
      0b00110110 => // RL  A
      0b00110111 => // RLC A
      0b00111000 => // NOT B
      0b00111001 => // NEG B
      0b00111010 => // INC B
      0b00111011 => // DEC B
      0b00111100 => // RR  B
      0b00111101 => // RRC B
      0b00111110 => // RL  B
      0b00111111 => // RLC B

      0b01
      0b10

// 01 yyy opr   opr  A,y
// 10 yyy opr   opr  B,y

// ALU-OP
// 000 ADD
// 001 ADDC
// 010 SUB
// 011 SUBC
// 100 AND
// 101 OR
// 110 XOR
// 111 CMP

// ALU-SRC
// 000 AB
// 001 addr
// 010 addr+X
// 011 addr+Y
// 100 byte
// 101 (addr)
// 110 (addr)+X
// 111 (addr)+Y

      0b11000000 => instructions.insert(byte, ld_reg_byte(Register::A)),
      0b11000001 => instructions.insert(byte, ld_reg_reg(Register::A, Register::B)),
      0b11000010 => instructions.insert(byte, ld_reg_reg(Register::A, Register::X)),
      0b11000011 => instructions.insert(byte, ld_reg_reg(Register::A, Register::Y)),
      0b11000100 => instructions.insert(byte, ld_reg_reg(Register::B, Register::A)),
      0b11000101 => instructions.insert(byte, ld_reg_byte(Register::B),
      0b11000110 => instructions.insert(byte, ld_reg_reg(Register::B, Register::X)),
      0b11000111 => instructions.insert(byte, ld_reg_reg(Register::B, Register::Y)),
      0b11001000 => instructions.insert(byte, ld_reg_reg(Register::X, Register::A)),
      0b11001001 => instructions.insert(byte, ld_reg_reg(Register::X, Register::B)),
      0b11001010 => instructions.insert(byte, ld_reg_byte(Register::X)),
      0b11001011 => instructions.insert(byte, ld_reg_reg(Register::X, Register::Y)),
      0b11001100 => instructions.insert(byte, ld_reg_reg(Register::Y, Register::A)),
      0b11001101 => instructions.insert(byte, ld_reg_reg(Register::Y, Register::B)),
      0b11001110 => instructions.insert(byte, ld_reg_reg(Register::Y, Register::X)),
      0b11001111 => instructions.insert(byte, ld_reg_byte(Register::Y)),
      0b11010000 => instructions.insert(byte, ld_reg_mem(Register::X, Memory::Address)),
      0b11010001 => instructions.insert(byte, ld_reg_mem(Register::X, Memory::IndexedAddress(Register::Y))),
      0b11010010 => instructions.insert(byte, ld_reg_mem(Register::X, Memory::IndirectAddress)),
      0b11010011 => instructions.insert(byte, ld_reg_mem(Register::X, Memory::IndirectIndexedAddress(Register::Y))),
      0b11010100 => instructions.insert(byte, ld_reg_mem(Register::Y, Memory::Address)),
      0b11010101 => instructions.insert(byte, ld_reg_mem(Register::Y, Memory::IndexedAddress(Register::X))),
      0b11010110 => instructions.insert(byte, ld_reg_mem(Register::Y, Memory::IndirectAddress)),
      0b11010111 => instructions.insert(byte, ld_reg_mem(Register::Y, Memory::IndirectIndexedAddress(Register::X))),
      0b11011000 => instructions.insert(byte, ld_mem_reg(Register::X, Memory::Address)),
      0b11011001 => instructions.insert(byte, ld_mem_reg(Register::X, Memory::IndexedAddress(Register::Y))),
      0b11011010 => instructions.insert(byte, ld_mem_reg(Register::X, Memory::IndirectAddress)),
      0b11011011 => instructions.insert(byte, ld_mem_reg(Register::X, Memory::IndirectIndexedAddress(Register::Y))),
      0b11011100 => instructions.insert(byte, ld_mem_reg(Register::Y, Memory::Address)),
      0b11011101 => instructions.insert(byte, ld_mem_reg(Register::Y, Memory::IndexedAddress(Register::X))),
      0b11011110 => instructions.insert(byte, ld_mem_reg(Register::Y, Memory::IndirectAddress)),
      0b11011111 => instructions.insert(byte, ld_mem_reg(Register::Y, Memory::IndirectIndexedAddress(Register::X))),
      0b11100000 => instructions.insert(byte, pop(Register::A)),
      0b11100001 => instructions.insert(byte, ld_reg_mem(Register::A, Memory::Address)),
      0b11100010 => instructions.insert(byte, ld_reg_mem(Register::A, Memory::IndexedAddress(Register::X))),
      0b11100011 => instructions.insert(byte, ld_reg_mem(Register::A, Memory::IndexedAddress(Register::Y))),
      0b11100100 => instructions.insert(byte, pop(Register::B)),
      0b11100101 => instructions.insert(byte, ld_reg_mem(Register::A, Memory::IndirectAddress)),
      0b11100110 => instructions.insert(byte, ld_reg_mem(Register::A, Memory::IndirectIndexedAddress(Register::X))),
      0b11100111 => instructions.insert(byte, ld_reg_mem(Register::A, Memory::IndirectIndexedAddress(Register::Y))),
      0b11101000 => instructions.insert(byte, pop(Register::X)),
      0b11101001 => instructions.insert(byte, ld_mem_reg(Register::A, Memory::Address)),
      0b11101010 => instructions.insert(byte, ld_mem_reg(Register::A, Memory::IndexedAddress(Register::X))),
      0b11101011 => instructions.insert(byte, ld_mem_reg(Register::A, Memory::IndexedAddress(Register::Y))),
      0b11101100 => instructions.insert(byte, pop(Register::Y)),
      0b11101101 => instructions.insert(byte, ld_mem_reg(Register::A, Memory::IndirectAddress)),
      0b11101110 => instructions.insert(byte, ld_mem_reg(Register::A, Memory::IndirectIndexedAddress(Register::X))),
      0b11101111 => instructions.insert(byte, ld_mem_reg(Register::A, Memory::IndirectIndexedAddress(Register::Y))),
      0b11110000 => instructions.insert(byte, push(Register::A)),
      0b11110001 => instructions.insert(byte, ld_reg_mem(Register::B, Memory::Address)),
      0b11110010 => instructions.insert(byte, ld_reg_mem(Register::B, Memory::IndexedAddress(Register::X))),
      0b11110011 => instructions.insert(byte, ld_reg_mem(Register::B, Memory::IndexedAddress(Register::Y))),
      0b11110100 => instructions.insert(byte, push(Register::B)),
      0b11110101 => instructions.insert(byte, ld_reg_mem(Register::B, Memory::IndirectAddress)),
      0b11110110 => instructions.insert(byte, ld_reg_mem(Register::B, Memory::IndirectIndexedAddress(Register::X))),
      0b11110111 => instructions.insert(byte, ld_reg_mem(Register::B, Memory::IndirectIndexedAddress(Register::Y))),
      0b11111000 => instructions.insert(byte, push(Register::X)),
      0b11111001 => instructions.insert(byte, ld_mem_reg(Register::B, Memory::Address)),
      0b11111010 => instructions.insert(byte, ld_mem_reg(Register::B, Memory::IndexedAddress(Register::X))),
      0b11111011 => instructions.insert(byte, ld_mem_reg(Register::B, Memory::IndexedAddress(Register::Y))),
      0b11111100 => instructions.insert(byte, push(Register::Y)),
      0b11111101 => instructions.insert(byte, ld_mem_reg(Register::B, Memory::IndirectAddress)),
      0b11111110 => instructions.insert(byte, ld_mem_reg(Register::B, Memory::IndirectIndexedAddress(Register::X))),
      0b11111111 => instructions.insert(byte, ld_mem_reg(Register::B, Memory::IndirectIndexedAddress(Register::Y))),
    }

    for value in instructions.values_mut() {
      match &mut value[&0] {
        Micro::Static(c) => {
          c.ProgramCounter.Count = ProgramCounterCount::Increment;
        },
        Micro::Flag(_, t, f) => {
          t.ProgramCounter.Count = ProgramCounterCount::Increment;
          f.ProgramCounter.Count = ProgramCounterCount::Increment;
        },
      }
    }

    First {
      instructions: instructions
    }
  }
}

impl instructions::Set for First {
  fn instruction(&self, byte: u8) -> Vec<Micro> {
    self.instructions[&byte].clone();
  }
}
