
#[macro_use]
extern crate nom;
extern crate sdl2;


mod hash_map;
mod math;
mod error;

mod assembler;

mod bus;
mod control;

mod components;
mod memory_controller;
mod io;

mod instructions;

mod cpu;

mod ui;


use crate::cpu::Cpu;
use crate::instructions::set::Set;


fn main() {
  let hz: u64 = match option_env!("CLK") {
    Some(v) => u64::from_str_radix(v, 10).unwrap(),
    None => 5,
  };
  println!("CLK: {}", hz);

  let cpu = Cpu::new(hz, Box::new(Set::new()));

  ui::run(cpu).unwrap();
}
