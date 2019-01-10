
mod hash_map;
mod math;
mod error;

mod bus;
mod control;

mod program_counter;
mod stack_pointer;
mod register;
mod h_register;
mod l_register;
mod memory;
mod flags_register;
mod alu;

mod control_logic;
mod instructions;

mod cpu;

use crate::cpu::Cpu;

fn main() {
  let cpu = Cpu::new();
}
