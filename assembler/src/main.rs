
#[macro_use]
extern crate nom;

#[macro_use]
mod error;
mod symbols;
mod parser;
mod assembler;
mod preprocessor;

use std::env;
use std::io;
use std::io::prelude::*;

use crate::error::Result;


fn assemble(filename: &str) -> Result<()> {
  let file = preprocessor::preprocess(&filename)?;
  let symbols = parser::parse(&file)?;
  let words = assembler::assemble(symbols)?;

  let bytes: Vec<u8> = words.into_iter()
    .map(|word| {
      let [l, h] = u16::to_le_bytes(word);
      vec![l, h]
    })
    .flatten()
    .collect();
  io::stdout().write(bytes.as_slice())?;

  Ok(())
}

fn main() {
  let mut args = env::args();
  let filename = match args.nth(1) {
    None => panic!("Usage:\n\tassembler <filename>"),
    Some(filename) => filename,
  };

  if let Err(error) = assemble(&filename) {
    eprintln!("Error:\n\t{}", error);
    std::process::exit(1)
  }
}
