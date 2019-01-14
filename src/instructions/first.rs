
use std::collections::HashMap;

use crate::instructions;
use crate::instructions::Micro;
use crate::control::{
  Control,
  Read,
  Write,
  ReadWrite,
  IncDec,
  Flag,
  AluSelect,
  AluOperation,
  AluInput,
  AluRotateDirection,
};


#[derive(PartialEq, Eq)]
enum Register {
  A,
  B,
  X,
  Y,
}
enum Address {
  Offset,
  Address,
  IndexedAddress(Register),
  IndirectAddress,
  IndirectIndexedAddress(Register),
}
enum Argument {
  Byte,
  Register(Register),
  Address(Address),
}
enum Operation {
  Add,
  AddCarry,
  Sub,
  SubCarry,
  Compare,
  And,
  Or,
  Xor,
}
enum Unary {
  Not,
  Negate,
  Increment,
  Decrement,
  RotateRight,
  RotateRightCarry,
  RotateLeft,
  RotateLeftCarry,
}
enum JmpCondition {
  None,
  Flag(Flag, bool),
}


fn set_register(c: &mut Control, register: &Register, value: ReadWrite) {
  match register {
    Register::A => c.A.Data = value,
    Register::B => c.B.Data = value,
    Register::X => c.X.Data = value,
    Register::Y => c.Y.Data = value,
  }
}

fn alu_add(c: &mut Control, with_carry: bool, sign_extend: bool) {
  c.Alu.TempSelect = AluSelect::Value;
  c.Alu.Operation = AluOperation::Add {
    SignExtend: sign_extend,
    Carry: if with_carry {
      AluSelect::Value
    } else {
      AluSelect::Zero
    },
  };
}
fn alu_sub(c: &mut Control, with_carry: bool) {
  c.Alu.TempSelect = AluSelect::Invert;
  c.Alu.Operation = AluOperation::Add {
    SignExtend: false,
    Carry: if with_carry {
      AluSelect::Invert
    } else {
      AluSelect::One
    },
  };
}
fn alu_and(c: &mut Control) {
  c.Alu.TempSelect = AluSelect::Value;
  c.Alu.Operation = AluOperation::And;
}
fn alu_or(c: &mut Control) {
  c.Alu.TempSelect = AluSelect::Value;
  c.Alu.Operation = AluOperation::Or;
}
fn alu_xor(c: &mut Control) {
  c.Alu.TempSelect = AluSelect::Value;
  c.Alu.Operation = AluOperation::Xor;
}
fn alu_inc(c: &mut Control) {
  c.Alu.TempSelect = AluSelect::Zero;
  c.Alu.Operation = AluOperation::Add {
    SignExtend: false,
    Carry: AluSelect::One,
  };
}
fn alu_dec(c: &mut Control) {
  c.Alu.TempSelect = AluSelect::One;
  c.Alu.Operation = AluOperation::Add {
    SignExtend: false,
    Carry: AluSelect::One,
  };
}
fn alu_neg(c: &mut Control) {
  c.Alu.Input = AluInput::Zero;
  c.Alu.TempSelect = AluSelect::Invert;
  c.Alu.Operation = AluOperation::Add {
    SignExtend: false,
    Carry: AluSelect::One,
  };
}
fn alu_not(c: &mut Control) {
  c.Alu.Input = AluInput::Zero;
  c.Alu.TempSelect = AluSelect::Invert;
  c.Alu.Operation = AluOperation::Add {
    SignExtend: false,
    Carry: AluSelect::Zero,
  };
}
fn alu_rotate(c: &mut Control, direction: AluRotateDirection, carry: bool) {
  c.Alu.TempSelect = AluSelect::Zero;
  c.Alu.Operation = AluOperation::Rotate {
    Direction: direction,
    Carry: carry,
  };
}

fn addr() -> (usize, Vec<Control>) {
  let mut c = vec![Control::new(), Control::new(), Control::new()];
  
  c[0].ProgramCounter.Addr = ReadWrite::Write;
  c[0].Memory.Data = ReadWrite::Write;
  c[0].AddressRegister.DataH = Read::Read;

  c[1].ProgramCounter.Count = IncDec::Increment;
  c[1].ProgramCounter.Addr =  ReadWrite::Write;
  c[1].Memory.Data = ReadWrite::Write;
  c[1].AddressRegister.DataL = Read::Read;

  c[2].ProgramCounter.Count = IncDec::Increment;
  c[2].AddressRegister.Addr = Write::Write;

  (2, c)
}
fn idx_addr(index: &Register) -> (usize, Vec<Control>) {
  let mut c = vec![Control::new(), Control::new(), Control::new(), Control::new()];

  c[0].ProgramCounter.Addr = ReadWrite::Write;
  c[0].Memory.Data = ReadWrite::Write;
  c[0].AddressRegister.DataH = Read::Read;

  c[1].ProgramCounter.Count = IncDec::Increment;
  c[1].ProgramCounter.Addr = ReadWrite::Write;
  c[1].Memory.Data = ReadWrite::Write;
  c[1].AddressRegister.DataL = Read::Read;

  c[2].ProgramCounter.Count = IncDec::Increment;
  set_register(&mut c[2], index, ReadWrite::Write);
  c[2].Alu.Temp = Read::Read;
  c[2].AddressRegister.Addr = Write::Write;
  c[2].Alu.Input = AluInput::Addr;
  alu_add(&mut c[2], false, false);
  c[2].Alu.Output = Write::Write;

  c[3].Alu.Addr = Write::Write;

  (3, c)
}
fn ind_addr() -> (usize, Vec<Control>) {
  let mut c = vec![Control::new(), Control::new(), Control::new(), Control::new(), Control::new()];

  c[0].ProgramCounter.Addr = ReadWrite::Write;
  c[0].Memory.Data = ReadWrite::Write;
  c[0].AddressRegister.DataH = Read::Read;

  c[1].ProgramCounter.Count = IncDec::Increment;
  c[1].ProgramCounter.Addr = ReadWrite::Write;
  c[1].Memory.Data = ReadWrite::Write;
  c[1].AddressRegister.DataL = Read::Read;

  c[2].ProgramCounter.Count = IncDec::Increment;
  c[2].AddressRegister.Addr = Write::Write;
  c[2].Alu.Input = AluInput::Addr;
  alu_inc(&mut c[2]);
  c[2].Alu.Output = Write::Write;
  c[2].Memory.Data = ReadWrite::Write;
  c[2].AddressRegister.DataH = Read::Read;

  c[3].Alu.Addr = Write::Write;
  c[3].Memory.Data = ReadWrite::Write;
  c[3].AddressRegister.DataL = Read::Read;

  c[4].AddressRegister.Addr = Write::Write;

  (4, c)
}
fn ind_idx_addr(index: &Register) -> (usize, Vec<Control>) {
  let mut c = vec![Control::new(), Control::new(), Control::new(), Control::new(), Control::new()];

  c[0].ProgramCounter.Addr = ReadWrite::Write;
  c[0].Memory.Data = ReadWrite::Write;
  c[0].AddressRegister.DataH = Read::Read;

  c[1].ProgramCounter.Count = IncDec::Increment;
  c[1].ProgramCounter.Addr = ReadWrite::Write;
  c[1].Memory.Data = ReadWrite::Write;
  c[1].AddressRegister.DataL = Read::Read;

  c[2].ProgramCounter.Count = IncDec::Increment;
  set_register(&mut c[2], index, ReadWrite::Write);
  c[2].Alu.Temp = Read::Read;
  c[2].AddressRegister.Addr = Write::Write;
  c[2].Alu.Input = AluInput::Addr;
  alu_add(&mut c[2], false, false);
  c[2].Alu.Output = Write::Write;
  c[2].Memory.Data = ReadWrite::Write;
  c[2].AddressRegister.DataH = Read::Read;

  c[3].Alu.Addr = Write::Write;
  c[3].Memory.Data = ReadWrite::Write;
  c[3].AddressRegister.DataL = Read::Read;

  c[4].AddressRegister.Addr = Write::Write;

  (4, c)
}


fn nop() -> Micro {
  Micro::Compress(vec![Control::new()])
}

fn halt() -> Micro {
  let mut c = Control::new();
  c.Instruction.Halt = true;
  Micro::Code(vec![c])
}

fn interrupt(handler: &[u16; 2], halt: bool) -> Micro {
  let mut c = vec![];

  c.push(Control::new());
  c[0].StackPointer.Addr = Write::Write;
  c[0].FlagsRegister.Data = ReadWrite::Write;
  c[0].Memory.Data = ReadWrite::Read;

  c.push(Control::new());
  c[1].StackPointer.Count = IncDec::Decrement;
  c[1].StackPointer.Addr = Write::Write;
  c[1].ProgramCounter.DataL = Write::Write;
  c[1].Memory.Data = ReadWrite::Read;

  c.push(Control::new());
  c[2].StackPointer.Count = IncDec::Decrement;
  c[2].StackPointer.Addr = Write::Write;
  c[2].ProgramCounter.DataH = Write::Write;
  c[2].Memory.Data = ReadWrite::Read;

  c.push(Control::new());
  c[3].StackPointer.Count = IncDec::Decrement;
  c[3].Instruction.Vector = Some(handler[0]);
  c[3].Memory.Data = ReadWrite::Write;
  c[3].AddressRegister.DataH = Read::Read;

  c.push(Control::new());
  c[4].Instruction.Vector = Some(handler[1]);
  c[4].Memory.Data = ReadWrite::Write;
  c[4].AddressRegister.DataL = Read::Read;

  c.push(Control::new());
  c[5].AddressRegister.Addr = Write::Write;
  c[5].Alu.Input = AluInput::Addr;
  alu_inc(&mut c[5]);
  c[5].Alu.Output = Write::Write;
  c[5].Memory.Data = ReadWrite::Write;
  c[5].AddressRegister.DataH = Read::Read;

  c.push(Control::new());
  c[6].Alu.Addr = Write::Write;
  c[6].Memory.Data = ReadWrite::Write;
  c[6].AddressRegister.DataL = Read::Read;

  c.push(Control::new());
  c[7].AddressRegister.Addr = Write::Write;
  c[7].ProgramCounter.Addr = ReadWrite::Read;
  c[7].Instruction.Halt = halt;

  Micro::Code(c)
}

fn set_flag(flag: Flag, value: bool) -> Micro {
  let mut c = Control::new();
  match flag {
    Flag::C => c.FlagsRegister.C = Some(value),
    Flag::I => c.FlagsRegister.I = Some(value),
    _ => panic!("Can not call set_flag on {:?}.", flag),
  }
  Micro::Code(vec![c])
}

fn ret() -> Micro {
  let mut c = vec![Control::new(), Control::new(), Control::new()];

  c[0].StackPointer.Count = IncDec::Increment;
  c[0].StackPointer.Addr = Write::Write;
  c[0].Memory.Data = ReadWrite::Write;
  c[0].AddressRegister.DataH = Read::Read;

  c[1].StackPointer.Count = IncDec::Increment;
  c[1].StackPointer.Addr = Write::Write;
  c[1].Memory.Data = ReadWrite::Write;
  c[1].AddressRegister.DataL = Read::Read;

  c[2].AddressRegister.Addr = Write::Write;
  c[2].ProgramCounter.Addr = ReadWrite::Read;

  Micro::Code(c)
}

fn reti() -> Micro {
  let mut c = vec![Control::new(), Control::new(), Control::new(), Control::new()];

  c[0].StackPointer.Count = IncDec::Increment;
  c[0].StackPointer.Addr = Write::Write;
  c[0].Memory.Data = ReadWrite::Write;
  c[0].AddressRegister.DataH = Read::Read;

  c[1].StackPointer.Count = IncDec::Increment;
  c[1].StackPointer.Addr = Write::Write;
  c[1].Memory.Data = ReadWrite::Write;
  c[1].AddressRegister.DataL = Read::Read;

  c[2].StackPointer.Count = IncDec::Increment;
  c[2].StackPointer.Addr = Write::Write;
  c[2].Memory.Data = ReadWrite::Write;
  c[2].FlagsRegister.Data = ReadWrite::Read;

  c[3].AddressRegister.Addr = Write::Write;
  c[3].ProgramCounter.Addr = ReadWrite::Read;

  Micro::Code(c)
}

fn call(target: Address) -> Micro {
  let (i, mut c) = match target {
    Address::Offset => panic!("Only JMP supports Offset."),
    Address::Address => addr(),
    Address::IndexedAddress(_) => panic!("IndexedAddress not yet implemented for CALL."),
    Address::IndirectAddress => ind_addr(),
    Address::IndirectIndexedAddress(index) => ind_idx_addr(&index),
  };

  c.insert(i, Control::new());
  c.insert(i, Control::new());

  c[i].StackPointer.Addr = Write::Write;
  c[i].ProgramCounter.DataL = Write::Write;
  c[i].Memory.Data = ReadWrite::Read;

  c[i+1].StackPointer.Count = IncDec::Decrement;
  c[i+1].StackPointer.Addr = Write::Write;
  c[i+1].ProgramCounter.DataH = Write::Write;
  c[i+1].Memory.Data = ReadWrite::Read;

  c[i+2].StackPointer.Count = IncDec::Decrement;
  c[i+2].ProgramCounter.Addr = ReadWrite::Read;

  Micro::Code(c)
}

fn jmp(condition: JmpCondition, target: Address) -> Micro {
  let mut c = vec![Control::new(), Control::new(), Control::new()];

  match target {
    Address::Offset => {
      c[0].ProgramCounter.Addr = ReadWrite::Write;
      c[0].Memory.Data = ReadWrite::Write;
      c[0].Alu.Temp = Read::Read;

      c[1].ProgramCounter.Addr = ReadWrite::Write;
      c[1].Alu.Input = AluInput::Addr;
      alu_add(&mut c[1], false, true);
      c[1].Alu.Output = Write::Write;

      c[2].Alu.Addr = Write::Write;
      c[2].ProgramCounter.Addr = ReadWrite::Read;
    },
    Address::Address => {
      c[0].ProgramCounter.Addr = ReadWrite::Write;
      c[0].Memory.Data = ReadWrite::Write;
      c[0].AddressRegister.DataH = Read::Read;

      c[1].ProgramCounter.Addr = ReadWrite::Write;
      c[1].Memory.Data = ReadWrite::Write;
      c[1].AddressRegister.DataL = Read::Read;

      c[2].AddressRegister.Addr = Write::Write;
      c[2].ProgramCounter.Addr = ReadWrite::Read;
    },
    _ => panic!("JMP only supports Offset and Address."),
  }

  match condition {
    JmpCondition::None => Micro::Code(c),
    JmpCondition::Flag(flag, value) => {
      let n = Box::new(Micro::Compress(vec![Control::new()]));
      if value {
        Micro::Branch(flag, Box::new(Micro::Code(c)), n)
      } else {
        Micro::Branch(flag, n, Box::new(Micro::Code(c)))
      }
    },
  }
}

fn operation(register: Register, operation: Operation, argument: Argument) -> Micro {
  let (i, mut c) = match argument {
    Argument::Byte => {
      let mut c = vec![Control::new(), Control::new(), Control::new()];

      c[0].ProgramCounter.Addr = ReadWrite::Write;
      c[0].Memory.Data = ReadWrite::Write;
      c[0].Alu.Temp = Read::Read;

      c[1].ProgramCounter.Count = IncDec::Increment;
      set_register(&mut c[1], &register, ReadWrite::Write);
      c[1].Alu.Input = AluInput::Data;
      c[1].Alu.Output = Write::Write;

      c[2].Alu.Data = Write::Write;
      set_register(&mut c[2], &register, ReadWrite::Read);

      (1, c)
    },
    Argument::Register(r) => {
      let mut c = vec![Control::new(), Control::new(), Control::new()];

      c[0].ProgramCounter.Addr = ReadWrite::Write;
      set_register(&mut c[0], &r, ReadWrite::Write);
      c[0].Alu.Temp = Read::Read;

      set_register(&mut c[1], &register, ReadWrite::Write);
      c[1].Alu.Input = AluInput::Data;
      c[1].Alu.Output = Write::Write;

      c[2].Alu.Data = Write::Write;
      set_register(&mut c[2], &register, ReadWrite::Read);

      (1, c)
    },
    Argument::Address(a) => {
      let (i, mut c) = match a {
        Address::Offset => panic!("Only JMP supports Offset."),
        Address::Address => addr(),
        Address::IndexedAddress(index) => idx_addr(&index),
        Address::IndirectAddress => ind_addr(),
        Address::IndirectIndexedAddress(index) => ind_idx_addr(&index),
      };

      c.push(Control::new());
      c.push(Control::new());

      c[i].Memory.Data = ReadWrite::Write;
      c[i].Alu.Temp = Read::Read;

      c[i+1].ProgramCounter.Count = IncDec::Increment;
      set_register(&mut c[i+1], &register, ReadWrite::Write);
      c[i+1].Alu.Input = AluInput::Data;
      c[i+1].Alu.Output = Write::Write;

      c[i+2].Alu.Data = Write::Write;
      set_register(&mut c[i+2], &register, ReadWrite::Read);

      (i, c)
    },
  };

  match operation {
    Operation::Add      => alu_add(&mut c[i], false, false),
    Operation::AddCarry => alu_add(&mut c[i], true, false),
    Operation::Sub      => alu_sub(&mut c[i], false),
    Operation::SubCarry => alu_sub(&mut c[i], true),
    Operation::And      => alu_and(&mut c[i]),
    Operation::Or       => alu_or(&mut c[i]),
    Operation::Xor      => alu_xor(&mut c[i]),
    Operation::Compare  => {
      alu_sub(&mut c[i], false);
      c[i].Alu.Output = Write::None;
    },
  }
  c[i].FlagsRegister.Update = Read::Read;

  Micro::Code(c)
}
fn unary(register: Register, operation: Unary) -> Micro {
  let mut c = vec![Control::new(), Control::new()];

  set_register(&mut c[0], &register, ReadWrite::Write);
  c[0].Alu.Input = AluInput::Data;
  match operation {
    Unary::Increment => alu_inc(&mut c[0]),
    Unary::Decrement => alu_dec(&mut c[0]),
    Unary::RotateRight => alu_rotate(&mut c[0], AluRotateDirection::Right, false),
    Unary::RotateRightCarry => alu_rotate(&mut c[0], AluRotateDirection::Right, true),
    Unary::RotateLeft => alu_rotate(&mut c[0], AluRotateDirection::Left, false),
    Unary::RotateLeftCarry => alu_rotate(&mut c[0], AluRotateDirection::Left, true),
    Unary::Not | Unary::Negate => panic!("Use unary_t for the NOT and NEG operators."),
  }
  c[0].Alu.Output = Write::Write;
  c[0].FlagsRegister.Update = Read::Read;

  c[1].Alu.Data = Write::Write;
  set_register(&mut c[1], &register, ReadWrite::Read);

  Micro::Code(c)
}
fn unary_t(register: Register, operation: Unary) -> Micro {
  let mut c = vec![Control::new(), Control::new(), Control::new()];

  set_register(&mut c[0], &register, ReadWrite::Write);
  c[0].Alu.Temp = Read::Read;

  match operation {
    Unary::Not => alu_not(&mut c[1]),
    Unary::Negate => alu_neg(&mut c[1]),
    _ => panic!("Only use unary_t for the NOT and NEG operators."),
  }
  c[1].Alu.Output = Write::Write;
  c[1].FlagsRegister.Update = Read::Read;

  c[2].Alu.Data = Write::Write;
  set_register(&mut c[2], &register, ReadWrite::Read);

  Micro::Code(c)
}

fn push(src: Register) -> Micro {
  let mut c = vec![Control::new(), Control::new()];

  c[0].StackPointer.Addr = Write::Write;
  c[0].Memory.Data = ReadWrite::Write;
  set_register(&mut c[0], &src, ReadWrite::Read);

  c[1].StackPointer.Count = IncDec::Decrement;

  Micro::Compress(c)
}
fn pop(dest: Register) -> Micro {
  let mut c = Control::new();

  c.StackPointer.Count = IncDec::Increment;
  c.StackPointer.Addr = Write::Write;
  set_register(&mut c, &dest, ReadWrite::Write);
  c.Memory.Data = ReadWrite::Read;

  Micro::Code(vec![c])
}

fn ld_reg_byte(dest: Register) -> Micro {
  let mut c = vec![Control::new(), Control::new()];

  c[0].ProgramCounter.Addr = ReadWrite::Write;
  c[0].Memory.Data = ReadWrite::Write;
  set_register(&mut c[0], &dest, ReadWrite::Read);

  c[1].ProgramCounter.Count = IncDec::Increment;

  Micro::Compress(c)
}

fn ld_reg_reg(dest: Register, src: Register) -> Micro {
  if dest == src {
    panic!("ld_reg_reg can not have identical dest and src arguments. Use ld_reg_byte instead.");
  }
  let mut c = Control::new();
  set_register(&mut c, &dest, ReadWrite::Read);
  set_register(&mut c, &src, ReadWrite::Write);
  Micro::Code(vec![c])
}

fn ld_reg_mem_(register: Register, memory: Address, r_value: ReadWrite, m_value: ReadWrite) -> Micro {
  let (i, mut c) = match memory {
    Address::Offset => panic!("Only JMP supports Offset."),
    Address::Address => addr(),
    Address::IndexedAddress(index) => idx_addr(&index),
    Address::IndirectAddress => ind_addr(),
    Address::IndirectIndexedAddress(index) => ind_idx_addr(&index),
  };

  c[i].Memory.Data = m_value;
  set_register(&mut c[i], &register, r_value);

  Micro::Code(c)
}
fn ld_reg_mem(dest: Register, src: Address) -> Micro {
  ld_reg_mem_(dest, src, ReadWrite::Read, ReadWrite::Write)
}
fn ld_mem_reg(src: Register, dest: Address) -> Micro {
  ld_reg_mem_(src, dest, ReadWrite::Write, ReadWrite::Read)
}


pub struct First {
  instructions: HashMap<u8, Micro>,
}

impl First {
  pub fn new() -> First {
    let mut instructions = hash_map!{
      0b00000000 => nop(),
      0b00000001 => halt(), // HLT
      0b00000010 => interrupt(&crate::memory::BREAK_HANDLER, true), // BRK
      0b00000011 => interrupt(&crate::memory::INTERRUPT_HANDLER, false), // INT
      0b00000100 => set_flag(Flag::C, false),
      0b00000101 => set_flag(Flag::C, true),
      0b00000110 => set_flag(Flag::I, false),
      0b00000111 => set_flag(Flag::I, true),
      0b00001000 => call(Address::Address),
      0b00001001 => call(Address::IndirectAddress),
      0b00001010 => call(Address::IndirectIndexedAddress(Register::X)),
      0b00001011 => call(Address::IndirectIndexedAddress(Register::Y)),
      0b00001100 => ret(),  // Return
      0b00001101 => reti(), // Return from interrupt
      0b00001110 => jmp(JmpCondition::None, Address::Address),
      0b00001111 => jmp(JmpCondition::None, Address::Offset),
      0b00010000 => jmp(JmpCondition::Flag(Flag::Z, false), Address::Offset), // JMP NZ,+byte
      0b00010001 => jmp(JmpCondition::Flag(Flag::Z, true),  Address::Offset), // JMP  Z,+byte
      0b00010010 => jmp(JmpCondition::Flag(Flag::C, false), Address::Offset), // JMP NC,+byte
      0b00010011 => jmp(JmpCondition::Flag(Flag::C, true),  Address::Offset), // JMP  C,+byte
      0b00010100 => jmp(JmpCondition::Flag(Flag::V, false), Address::Offset), // JMP NV,+byte
      0b00010101 => jmp(JmpCondition::Flag(Flag::V, true),  Address::Offset), // JMP  V,+byte
      0b00010110 => jmp(JmpCondition::Flag(Flag::S, false), Address::Offset), // JMP  P,+byte
      0b00010111 => jmp(JmpCondition::Flag(Flag::S, true),  Address::Offset), // JMP  N,+byte
      0b00011000 => unary(Register::X, Unary::Increment),
      0b00011001 => unary(Register::X, Unary::Decrement),
      0b00011010 => operation(Register::X, Operation::Add, Argument::Byte),
      0b00011011 => operation(Register::X, Operation::Sub, Argument::Byte),
      0b00011100 => unary(Register::Y, Unary::Increment),
      0b00011101 => unary(Register::Y, Unary::Decrement),
      0b00011110 => operation(Register::Y, Operation::Add, Argument::Byte),
      0b00011111 => operation(Register::Y, Operation::Sub, Argument::Byte),
      0b00100000 => operation(Register::X, Operation::Compare, Argument::Register(Register::A)),
      0b00100001 => operation(Register::X, Operation::Compare, Argument::Register(Register::B)),
      0b00100010 => operation(Register::X, Operation::Compare, Argument::Register(Register::Y)),
      0b00100011 => operation(Register::X, Operation::Compare, Argument::Byte),
      0b00100100 => operation(Register::X, Operation::Compare, Argument::Address(Address::Address)),
      0b00100101 => operation(Register::X, Operation::Compare, Argument::Address(Address::IndexedAddress(Register::Y))),
      0b00100110 => operation(Register::X, Operation::Compare, Argument::Address(Address::IndirectAddress)),
      0b00100111 => operation(Register::X, Operation::Compare, Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b00101000 => operation(Register::Y, Operation::Compare, Argument::Register(Register::A)),
      0b00101001 => operation(Register::Y, Operation::Compare, Argument::Register(Register::B)),
      0b00101010 => operation(Register::Y, Operation::Compare, Argument::Register(Register::X)),
      0b00101011 => operation(Register::Y, Operation::Compare, Argument::Byte),
      0b00101100 => operation(Register::Y, Operation::Compare, Argument::Address(Address::Address)),
      0b00101101 => operation(Register::Y, Operation::Compare, Argument::Address(Address::IndexedAddress(Register::X))),
      0b00101110 => operation(Register::Y, Operation::Compare, Argument::Address(Address::IndirectAddress)),
      0b00101111 => operation(Register::Y, Operation::Compare, Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b00110000 => unary_t(Register::A, Unary::Not),
      0b00110001 => unary_t(Register::A, Unary::Negate),
      0b00110010 => unary(Register::A, Unary::Increment),
      0b00110011 => unary(Register::A, Unary::Decrement),
      0b00110100 => unary(Register::A, Unary::RotateRight),
      0b00110101 => unary(Register::A, Unary::RotateRightCarry),
      0b00110110 => unary(Register::A, Unary::RotateLeft),
      0b00110111 => unary(Register::A, Unary::RotateLeftCarry),
      0b00111000 => unary_t(Register::B, Unary::Not),
      0b00111001 => unary_t(Register::B, Unary::Negate),
      0b00111010 => unary(Register::B, Unary::Increment),
      0b00111011 => unary(Register::B, Unary::Decrement),
      0b00111100 => unary(Register::B, Unary::RotateRight),
      0b00111101 => unary(Register::B, Unary::RotateRightCarry),
      0b00111110 => unary(Register::B, Unary::RotateLeft),
      0b00111111 => unary(Register::B, Unary::RotateLeftCarry),
      0b01000000 => operation(Register::A, Operation::Add,      Argument::Register(Register::B)),
      0b01000001 => operation(Register::A, Operation::AddCarry, Argument::Register(Register::B)),
      0b01000010 => operation(Register::A, Operation::Sub,      Argument::Register(Register::B)),
      0b01000011 => operation(Register::A, Operation::SubCarry, Argument::Register(Register::B)),
      0b01000100 => operation(Register::A, Operation::And,      Argument::Register(Register::B)),
      0b01000101 => operation(Register::A, Operation::Or,       Argument::Register(Register::B)),
      0b01000110 => operation(Register::A, Operation::Xor,      Argument::Register(Register::B)),
      0b01000111 => operation(Register::A, Operation::Compare,  Argument::Register(Register::B)),
      0b01001000 => operation(Register::A, Operation::Add,      Argument::Address(Address::Address)),
      0b01001001 => operation(Register::A, Operation::AddCarry, Argument::Address(Address::Address)),
      0b01001010 => operation(Register::A, Operation::Sub,      Argument::Address(Address::Address)),
      0b01001011 => operation(Register::A, Operation::SubCarry, Argument::Address(Address::Address)),
      0b01001100 => operation(Register::A, Operation::And,      Argument::Address(Address::Address)),
      0b01001101 => operation(Register::A, Operation::Or,       Argument::Address(Address::Address)),
      0b01001110 => operation(Register::A, Operation::Xor,      Argument::Address(Address::Address)),
      0b01001111 => operation(Register::A, Operation::Compare,  Argument::Address(Address::Address)),
      0b01010000 => operation(Register::A, Operation::Add,      Argument::Address(Address::IndexedAddress(Register::X))),
      0b01010001 => operation(Register::A, Operation::AddCarry, Argument::Address(Address::IndexedAddress(Register::X))),
      0b01010010 => operation(Register::A, Operation::Sub,      Argument::Address(Address::IndexedAddress(Register::X))),
      0b01010011 => operation(Register::A, Operation::SubCarry, Argument::Address(Address::IndexedAddress(Register::X))),
      0b01010100 => operation(Register::A, Operation::And,      Argument::Address(Address::IndexedAddress(Register::X))),
      0b01010101 => operation(Register::A, Operation::Or,       Argument::Address(Address::IndexedAddress(Register::X))),
      0b01010110 => operation(Register::A, Operation::Xor,      Argument::Address(Address::IndexedAddress(Register::X))),
      0b01010111 => operation(Register::A, Operation::Compare,  Argument::Address(Address::IndexedAddress(Register::X))),
      0b01011000 => operation(Register::A, Operation::Add,      Argument::Address(Address::IndexedAddress(Register::Y))),
      0b01011001 => operation(Register::A, Operation::AddCarry, Argument::Address(Address::IndexedAddress(Register::Y))),
      0b01011010 => operation(Register::A, Operation::Sub,      Argument::Address(Address::IndexedAddress(Register::Y))),
      0b01011011 => operation(Register::A, Operation::SubCarry, Argument::Address(Address::IndexedAddress(Register::Y))),
      0b01011100 => operation(Register::A, Operation::And,      Argument::Address(Address::IndexedAddress(Register::Y))),
      0b01011101 => operation(Register::A, Operation::Or,       Argument::Address(Address::IndexedAddress(Register::Y))),
      0b01011110 => operation(Register::A, Operation::Xor,      Argument::Address(Address::IndexedAddress(Register::Y))),
      0b01011111 => operation(Register::A, Operation::Compare,  Argument::Address(Address::IndexedAddress(Register::Y))),
      0b01100000 => operation(Register::A, Operation::Add,      Argument::Byte),
      0b01100001 => operation(Register::A, Operation::AddCarry, Argument::Byte),
      0b01100010 => operation(Register::A, Operation::Sub,      Argument::Byte),
      0b01100011 => operation(Register::A, Operation::SubCarry, Argument::Byte),
      0b01100100 => operation(Register::A, Operation::And,      Argument::Byte),
      0b01100101 => operation(Register::A, Operation::Or,       Argument::Byte),
      0b01100110 => operation(Register::A, Operation::Xor,      Argument::Byte),
      0b01100111 => operation(Register::A, Operation::Compare,  Argument::Byte),
      0b01101000 => operation(Register::A, Operation::Add,      Argument::Address(Address::IndirectAddress)),
      0b01101001 => operation(Register::A, Operation::AddCarry, Argument::Address(Address::IndirectAddress)),
      0b01101010 => operation(Register::A, Operation::Sub,      Argument::Address(Address::IndirectAddress)),
      0b01101011 => operation(Register::A, Operation::SubCarry, Argument::Address(Address::IndirectAddress)),
      0b01101100 => operation(Register::A, Operation::And,      Argument::Address(Address::IndirectAddress)),
      0b01101101 => operation(Register::A, Operation::Or,       Argument::Address(Address::IndirectAddress)),
      0b01101110 => operation(Register::A, Operation::Xor,      Argument::Address(Address::IndirectAddress)),
      0b01101111 => operation(Register::A, Operation::Compare,  Argument::Address(Address::IndirectAddress)),
      0b01110000 => operation(Register::A, Operation::Add,      Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b01110001 => operation(Register::A, Operation::AddCarry, Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b01110010 => operation(Register::A, Operation::Sub,      Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b01110011 => operation(Register::A, Operation::SubCarry, Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b01110100 => operation(Register::A, Operation::And,      Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b01110101 => operation(Register::A, Operation::Or,       Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b01110110 => operation(Register::A, Operation::Xor,      Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b01110111 => operation(Register::A, Operation::Compare,  Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b01111000 => operation(Register::A, Operation::Add,      Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b01111001 => operation(Register::A, Operation::AddCarry, Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b01111010 => operation(Register::A, Operation::Sub,      Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b01111011 => operation(Register::A, Operation::SubCarry, Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b01111100 => operation(Register::A, Operation::And,      Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b01111101 => operation(Register::A, Operation::Or,       Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b01111110 => operation(Register::A, Operation::Xor,      Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b01111111 => operation(Register::A, Operation::Compare,  Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b10000000 => operation(Register::B, Operation::Add,      Argument::Register(Register::A)),
      0b10000001 => operation(Register::B, Operation::AddCarry, Argument::Register(Register::A)),
      0b10000010 => operation(Register::B, Operation::Sub,      Argument::Register(Register::A)),
      0b10000011 => operation(Register::B, Operation::SubCarry, Argument::Register(Register::A)),
      0b10000100 => operation(Register::B, Operation::And,      Argument::Register(Register::A)),
      0b10000101 => operation(Register::B, Operation::Or,       Argument::Register(Register::A)),
      0b10000110 => operation(Register::B, Operation::Xor,      Argument::Register(Register::A)),
      0b10000111 => operation(Register::B, Operation::Compare,  Argument::Register(Register::A)),
      0b10001000 => operation(Register::B, Operation::Add,      Argument::Address(Address::Address)),
      0b10001001 => operation(Register::B, Operation::AddCarry, Argument::Address(Address::Address)),
      0b10001010 => operation(Register::B, Operation::Sub,      Argument::Address(Address::Address)),
      0b10001011 => operation(Register::B, Operation::SubCarry, Argument::Address(Address::Address)),
      0b10001100 => operation(Register::B, Operation::And,      Argument::Address(Address::Address)),
      0b10001101 => operation(Register::B, Operation::Or,       Argument::Address(Address::Address)),
      0b10001110 => operation(Register::B, Operation::Xor,      Argument::Address(Address::Address)),
      0b10001111 => operation(Register::B, Operation::Compare,  Argument::Address(Address::Address)),
      0b10010000 => operation(Register::B, Operation::Add,      Argument::Address(Address::IndexedAddress(Register::X))),
      0b10010001 => operation(Register::B, Operation::AddCarry, Argument::Address(Address::IndexedAddress(Register::X))),
      0b10010010 => operation(Register::B, Operation::Sub,      Argument::Address(Address::IndexedAddress(Register::X))),
      0b10010011 => operation(Register::B, Operation::SubCarry, Argument::Address(Address::IndexedAddress(Register::X))),
      0b10010100 => operation(Register::B, Operation::And,      Argument::Address(Address::IndexedAddress(Register::X))),
      0b10010101 => operation(Register::B, Operation::Or,       Argument::Address(Address::IndexedAddress(Register::X))),
      0b10010110 => operation(Register::B, Operation::Xor,      Argument::Address(Address::IndexedAddress(Register::X))),
      0b10010111 => operation(Register::B, Operation::Compare,  Argument::Address(Address::IndexedAddress(Register::X))),
      0b10011000 => operation(Register::B, Operation::Add,      Argument::Address(Address::IndexedAddress(Register::Y))),
      0b10011001 => operation(Register::B, Operation::AddCarry, Argument::Address(Address::IndexedAddress(Register::Y))),
      0b10011010 => operation(Register::B, Operation::Sub,      Argument::Address(Address::IndexedAddress(Register::Y))),
      0b10011011 => operation(Register::B, Operation::SubCarry, Argument::Address(Address::IndexedAddress(Register::Y))),
      0b10011100 => operation(Register::B, Operation::And,      Argument::Address(Address::IndexedAddress(Register::Y))),
      0b10011101 => operation(Register::B, Operation::Or,       Argument::Address(Address::IndexedAddress(Register::Y))),
      0b10011110 => operation(Register::B, Operation::Xor,      Argument::Address(Address::IndexedAddress(Register::Y))),
      0b10011111 => operation(Register::B, Operation::Compare,  Argument::Address(Address::IndexedAddress(Register::Y))),
      0b10100000 => operation(Register::B, Operation::Add,      Argument::Byte),
      0b10100001 => operation(Register::B, Operation::AddCarry, Argument::Byte),
      0b10100010 => operation(Register::B, Operation::Sub,      Argument::Byte),
      0b10100011 => operation(Register::B, Operation::SubCarry, Argument::Byte),
      0b10100100 => operation(Register::B, Operation::And,      Argument::Byte),
      0b10100101 => operation(Register::B, Operation::Or,       Argument::Byte),
      0b10100110 => operation(Register::B, Operation::Xor,      Argument::Byte),
      0b10100111 => operation(Register::B, Operation::Compare,  Argument::Byte),
      0b10101000 => operation(Register::B, Operation::Add,      Argument::Address(Address::IndirectAddress)),
      0b10101001 => operation(Register::B, Operation::AddCarry, Argument::Address(Address::IndirectAddress)),
      0b10101010 => operation(Register::B, Operation::Sub,      Argument::Address(Address::IndirectAddress)),
      0b10101011 => operation(Register::B, Operation::SubCarry, Argument::Address(Address::IndirectAddress)),
      0b10101100 => operation(Register::B, Operation::And,      Argument::Address(Address::IndirectAddress)),
      0b10101101 => operation(Register::B, Operation::Or,       Argument::Address(Address::IndirectAddress)),
      0b10101110 => operation(Register::B, Operation::Xor,      Argument::Address(Address::IndirectAddress)),
      0b10101111 => operation(Register::B, Operation::Compare,  Argument::Address(Address::IndirectAddress)),
      0b10110000 => operation(Register::B, Operation::Add,      Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b10110001 => operation(Register::B, Operation::AddCarry, Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b10110010 => operation(Register::B, Operation::Sub,      Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b10110011 => operation(Register::B, Operation::SubCarry, Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b10110100 => operation(Register::B, Operation::And,      Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b10110101 => operation(Register::B, Operation::Or,       Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b10110110 => operation(Register::B, Operation::Xor,      Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b10110111 => operation(Register::B, Operation::Compare,  Argument::Address(Address::IndirectIndexedAddress(Register::X))),
      0b10111000 => operation(Register::B, Operation::Add,      Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b10111001 => operation(Register::B, Operation::AddCarry, Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b10111010 => operation(Register::B, Operation::Sub,      Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b10111011 => operation(Register::B, Operation::SubCarry, Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b10111100 => operation(Register::B, Operation::And,      Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b10111101 => operation(Register::B, Operation::Or,       Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b10111110 => operation(Register::B, Operation::Xor,      Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b10111111 => operation(Register::B, Operation::Compare,  Argument::Address(Address::IndirectIndexedAddress(Register::Y))),
      0b11000000 => ld_reg_byte(Register::A),
      0b11000001 => ld_reg_reg(Register::A, Register::B),
      0b11000010 => ld_reg_reg(Register::A, Register::X),
      0b11000011 => ld_reg_reg(Register::A, Register::Y),
      0b11000100 => ld_reg_reg(Register::B, Register::A),
      0b11000101 => ld_reg_byte(Register::B),
      0b11000110 => ld_reg_reg(Register::B, Register::X),
      0b11000111 => ld_reg_reg(Register::B, Register::Y),
      0b11001000 => ld_reg_reg(Register::X, Register::A),
      0b11001001 => ld_reg_reg(Register::X, Register::B),
      0b11001010 => ld_reg_byte(Register::X),
      0b11001011 => ld_reg_reg(Register::X, Register::Y),
      0b11001100 => ld_reg_reg(Register::Y, Register::A),
      0b11001101 => ld_reg_reg(Register::Y, Register::B),
      0b11001110 => ld_reg_reg(Register::Y, Register::X),
      0b11001111 => ld_reg_byte(Register::Y),
      0b11010000 => ld_reg_mem(Register::X, Address::Address),
      0b11010001 => ld_reg_mem(Register::X, Address::IndexedAddress(Register::Y)),
      0b11010010 => ld_reg_mem(Register::X, Address::IndirectAddress),
      0b11010011 => ld_reg_mem(Register::X, Address::IndirectIndexedAddress(Register::Y)),
      0b11010100 => ld_reg_mem(Register::Y, Address::Address),
      0b11010101 => ld_reg_mem(Register::Y, Address::IndexedAddress(Register::X)),
      0b11010110 => ld_reg_mem(Register::Y, Address::IndirectAddress),
      0b11010111 => ld_reg_mem(Register::Y, Address::IndirectIndexedAddress(Register::X)),
      0b11011000 => ld_mem_reg(Register::X, Address::Address),
      0b11011001 => ld_mem_reg(Register::X, Address::IndexedAddress(Register::Y)),
      0b11011010 => ld_mem_reg(Register::X, Address::IndirectAddress),
      0b11011011 => ld_mem_reg(Register::X, Address::IndirectIndexedAddress(Register::Y)),
      0b11011100 => ld_mem_reg(Register::Y, Address::Address),
      0b11011101 => ld_mem_reg(Register::Y, Address::IndexedAddress(Register::X)),
      0b11011110 => ld_mem_reg(Register::Y, Address::IndirectAddress),
      0b11011111 => ld_mem_reg(Register::Y, Address::IndirectIndexedAddress(Register::X)),
      0b11100000 => pop(Register::A),
      0b11100001 => ld_reg_mem(Register::A, Address::Address),
      0b11100010 => ld_reg_mem(Register::A, Address::IndexedAddress(Register::X)),
      0b11100011 => ld_reg_mem(Register::A, Address::IndexedAddress(Register::Y)),
      0b11100100 => pop(Register::B),
      0b11100101 => ld_reg_mem(Register::A, Address::IndirectAddress),
      0b11100110 => ld_reg_mem(Register::A, Address::IndirectIndexedAddress(Register::X)),
      0b11100111 => ld_reg_mem(Register::A, Address::IndirectIndexedAddress(Register::Y)),
      0b11101000 => pop(Register::X),
      0b11101001 => ld_mem_reg(Register::A, Address::Address),
      0b11101010 => ld_mem_reg(Register::A, Address::IndexedAddress(Register::X)),
      0b11101011 => ld_mem_reg(Register::A, Address::IndexedAddress(Register::Y)),
      0b11101100 => pop(Register::Y),
      0b11101101 => ld_mem_reg(Register::A, Address::IndirectAddress),
      0b11101110 => ld_mem_reg(Register::A, Address::IndirectIndexedAddress(Register::X)),
      0b11101111 => ld_mem_reg(Register::A, Address::IndirectIndexedAddress(Register::Y)),
      0b11110000 => push(Register::A),
      0b11110001 => ld_reg_mem(Register::B, Address::Address),
      0b11110010 => ld_reg_mem(Register::B, Address::IndexedAddress(Register::X)),
      0b11110011 => ld_reg_mem(Register::B, Address::IndexedAddress(Register::Y)),
      0b11110100 => push(Register::B),
      0b11110101 => ld_reg_mem(Register::B, Address::IndirectAddress),
      0b11110110 => ld_reg_mem(Register::B, Address::IndirectIndexedAddress(Register::X)),
      0b11110111 => ld_reg_mem(Register::B, Address::IndirectIndexedAddress(Register::Y)),
      0b11111000 => push(Register::X),
      0b11111001 => ld_mem_reg(Register::B, Address::Address),
      0b11111010 => ld_mem_reg(Register::B, Address::IndexedAddress(Register::X)),
      0b11111011 => ld_mem_reg(Register::B, Address::IndexedAddress(Register::Y)),
      0b11111100 => push(Register::Y),
      0b11111101 => ld_mem_reg(Register::B, Address::IndirectAddress),
      0b11111110 => ld_mem_reg(Register::B, Address::IndirectIndexedAddress(Register::X)),
      0b11111111 => ld_mem_reg(Register::B, Address::IndirectIndexedAddress(Register::Y)),
    };

    for mut value in instructions.values_mut() {
      First::add_pc_inc(&mut value);
    }

    First {
      instructions: instructions
    }
  }

  fn add_pc_inc(m: &mut Micro) {
    match m {
      Micro::Code(c) |
      Micro::Compress(c) => {
        c[0].ProgramCounter.Count = IncDec::Increment;
      },
      Micro::Branch(_, t, f) => {
        First::add_pc_inc(t);
        First::add_pc_inc(f);
      },
    }
  }
}

impl instructions::Set for First {
  fn start(&self) -> u16 {
    crate::memory::START_ADDRESS
  }

  fn fetch(&self) -> Micro {
    let mut c = Control::new();
    c.ProgramCounter.Addr = ReadWrite::Write;
    c.Memory.Data = ReadWrite::Write;
    c.Instruction.Data = Read::Read;
    Micro::Code(vec![c])
  }

  fn get(&self, byte: u8) -> Micro {
    self.instructions[&byte].clone()
  }
}
