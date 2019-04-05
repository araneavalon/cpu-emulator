
#[macro_use]
extern crate nom;


mod hash_map;

mod components;
mod control;
mod cpu;

mod assembler;


fn main() {
	crate::assembler::parse("");
}
