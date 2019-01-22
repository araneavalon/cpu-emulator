
#[macro_use]
extern crate nom;

extern crate framebuffer;


mod hash_map;
mod math;
mod error;

mod bus;
mod control;

mod program_counter;
mod stack_pointer;
mod register;
mod address_register;
mod memory_controller;
mod io;
mod memory;
mod flags_register;
mod alu;

mod control_logic;
mod instructions;

mod cpu;

mod assembler;


use crate::cpu::Cpu;
use crate::instructions::set::Set;

fn main() {
  let hz: u64 = match option_env!("CLK") {
    Some(v) => u64::from_str_radix(v, 10).unwrap(),
    None => 5,
  };

  let mut cpu = Cpu::new(Box::new(Set::new()));
  cpu.run(hz).unwrap();
}
