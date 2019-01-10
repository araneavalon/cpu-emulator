
use std::collections::HashMap;

use crate::instructions;
use crate::control;

enum Flag {
	Z,
	C,
	V,
	S,
	I,
}

enum Register {
	A,
	B,
	X,
	Y,
}

enum Arg {
	Byte,
	Reg(Register),
}

mod call {
	enum Arg {
		Address,
		Indirect,
		IndexedIndirect(Register),
	}
}
mod jmp {
	enum Arg {
		Address,
		Offset,
	}
}

mod alu {
	enum Op {
		Add,
		Sub,
		Inc,
		Dec,

	}
}

enum I {
	Nop,
	Undefined,
	Set(Flag, u8),
	Call(call::Arg),
	Ret,
	RetI,
	Jmp(jmp::Arg),
	JmpC(bool, Flag, jmp::Arg),
	Alu(alu::Op, )
}





pub fn new() -> instructions::Set {
	let ins = hash_map!{
		0x00 => I::Nop,
		0x01 => I::Nop, // TODO ADD HLT CONTROL
		0x02 => I::Undefined,
		0x03 => I::Undefined,
		0x04 => I::Set(C, 0),
		0x05 => I::Set(C, 1),
		0x06 => I::Set(I, 0),
		0x07 => I::Set(I, 1),
		0x08 => I::Call(call::Arg::Address),                   // CALL addr
		0x09 => I::Call(call::Arg::Indirect),                  // CALL (addr)
		0x0a => I::Call(call::Arg::IndexedIndirect(Index::X)), // CALL (addr)+X
		0x0b => I::Call(call::Arg::IndexedIndirect(Index::X)), // CALL (addr)+Y
		0x0c => I::Ret,
		0x0d => I::RetI,
		0x0e => I::Jmp(jmp::Arg::Address),                  // JMP addr
		0x0f => I::Jmp(jmp::Arg::Offset),                   // JMP +byte
		0x10 => I::JmpC(false, Flag::Z, jmp::Arg::Offset),  // JMP  NZ,+byte
		0x11 => I::JmpC(true,  Flag::Z, jmp::Arg::Offset),  // JMP   Z,+byte
		0x12 => I::JmpC(false, Flag::C, jmp::Arg::Offset),  // JMP  NC,+byte
		0x13 => I::JmpC(true,  Flag::C, jmp::Arg::Offset),  // JMP   C,+byte
		0x14 => I::JmpC(false, Flag::V, jmp::Arg::Offset),  // JMP  NV,+byte
		0x15 => I::JmpC(true,  Flag::V, jmp::Arg::Offset),  // JMP   V,+byte
		0x16 => I::JmpC(false, Flag::S, jmp::Arg::Offset),  // JMP   P,+byte
		0x17 => I::JmpC(true,  Flag::S, jmp::Arg::Offset),  // JMP   N,+byte
		0x18 => 
		0x19 => 
		0x1a => 
		0x1b => 
		0x1c => 
		0x1d => 
		0x1e => 
		0x1f => 
		0x10 => 
    0x11 => 
    0x12 => 
    0x13 => 
    0x14 => 
    0x15 => 
    0x16 => 
    0x17 => 
    0x18 => 
    0x19 => 
    0x1a => 
    0x1b => 
    0x1c => 
    0x1d => 
    0x1e => 
    0x1f => 
	};

	instructions::Set::new(instructions)
}
