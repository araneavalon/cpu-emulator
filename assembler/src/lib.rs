
#[macro_use]
extern crate nom;

#[macro_use]
mod error;
mod symbols;
mod parser;
mod assembler;
mod preprocessor;

pub use error::{
  Error,
  Result,
};


pub fn from_string(file: &str) -> Result<Vec<u16>> {
  let symbols = parser::parse(file)?;
  let words = assembler::assemble(symbols)?;
  Ok(words)
}

pub fn from_string_bytes(file: &str) -> Result<Vec<u8>> {
  let words = from_string(file)?;
  let bytes = words.into_iter()
    .map(|word| {
      let [l, h] = u16::to_le_bytes(word);
      vec![l, h]
    })
    .flatten()
    .collect::<Vec<u8>>();
  Ok(bytes)
}

pub fn from_file(filename: &str) -> Result<Vec<u16>> {
  let file = preprocessor::preprocess(filename)?;
  from_string(&file)
}

pub fn from_file_bytes(filename: &str) -> Result<Vec<u8>> {
  let file = preprocessor::preprocess(filename)?;
  from_string_bytes(&file)
}
