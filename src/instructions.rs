
mod first;

use crate::control;


pub struct Set {
	instructions: HashMap<u8, Vec<control::Control>>,
}

impl Set {
	pub fn new(instructions: HashMap<u8, Vec<control::Control>>) -> Set {
		Set {
			instructions: instructions,
		}
	}

	pub fn fetch() -> Vec<control::Control> {
		let c = control::Control::new();
		c.ProgramCounter.Addr = control::ReadWrite::Write;
		c.Memory.Data = control::ReadWrite::Read;
		c.Instruction.Data = control::ReadWrite::Read;
		vec![c]
	}

	pub fn instruction(op: u8) -> Vec<control::Control> {
		let mut i = self.instructions[&op].clone();
		i[&0].ProgramCounter.Count = ProgramCounterCount::Increment;
		i
	}
}
