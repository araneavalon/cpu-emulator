
mod hash_map;
mod error;

mod bus;
mod addr;
mod connection;

mod clock;
mod mux;
mod input;
mod register;
mod program_counter;

mod ram;

mod cpu;

use crate::cpu::Cpu;


fn main() {
  let cpu = Cpu::new();
}
