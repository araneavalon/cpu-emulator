
mod hash_map;
mod math;
mod error;

mod bus;
mod control;

mod program_counter;
mod stack_pointer;
mod register;
mod address_register;
mod memory;
mod flags_register;
mod alu;

mod control_logic;
mod instructions;

mod cpu;

use std::io::{self, Read};

use crate::cpu::Cpu;
use crate::instructions::first::First;

fn main() -> io::Result<()> {
  let stdin = io::stdin();
  let mut cpu = Cpu::new(Box::new(First::new()));
  loop {
    match cpu.run(1000) {
      Ok(_) => (),
      Err(error) => panic!("{}", error),
    }
    println!("Press any key to resume execution... ");
    stdin.lock().read(&mut [0; 1])?;
    println!(" ... execution resumed.");
    cpu.resume();
  }
}
