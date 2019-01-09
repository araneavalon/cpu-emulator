
mod hash_map;
mod error;

mod bus;
mod control;

mod register;
mod program_counter;
mod memory;

mod cpu;

use crate::cpu::Cpu;

fn main() {
  let cpu = Cpu::new();
}
