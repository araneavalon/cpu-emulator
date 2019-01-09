
mod hash_map;
mod error;

mod bus;
mod control;

mod register;
mod program_counter;
mod stack_pointer;
mod memory;

mod control_logic;
mod cpu;

use crate::cpu::Cpu;

fn main() {
  let cpu = Cpu::new();
}
