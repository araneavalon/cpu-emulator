
use std::collections::HashMap;

mod error;
mod tokens;
mod parser;
mod assembler;

use crate::math::*;
use self::error::Error;
use self::tokens::Token;
use self::tokens::Value;
use self::tokens::Directive;


pub fn assemble(input: &str) -> Result<Vec<u8>, Error> {
  let mut tokens: Vec<Token> = parser::parse(input)?;
  let mut names: HashMap<String, Value> = HashMap::new();
  let mut out: Vec<u8> = Vec::new();

  println!("");
  println!("Tokens: {:?}\n", tokens);

  // Pass One: Calculate labels.
  let mut address = 0x0000;
  for token in tokens.iter() {
    match token {
      Token::Directive(directive) => {
        match directive {
          Directive::Section(addr) => address = *addr,
          Directive::Word(words) => address += (words.len() as u16) * 2,
          Directive::Byte(bytes) => address += bytes.len() as u16,
          Directive::Define(_, _) => (), // Don't do anything with these till pass two.
        }
      },
      Token::Label(label) => {
        names.insert(label.clone(), Value::Word(address));
      },
      Token::Op(op) => address += op.len(),
    }
  }

  println!("Labels: {:?}\n", names);

  // Pass Two: Everything Else
  let mut start: u16 = 0x0000;
  for token in tokens.iter_mut() {
    match token {
      Token::Directive(directive) => {
        match directive {
          Directive::Section(addr) => {
            if start == 0x0000 {
              start = *addr;
            }
            let padding = ((*addr - start) as i32) - (out.len() as i32);
            if padding < 0 {
              return Err(Error::SectionOverlap((out.len() as u16) + start, *addr));
            }
            for _ in 0..padding {
              out.push(0x00);
            }
          },
          Directive::Word(words) => {
            for expr in words.iter() {
              let word = expr.resolve_word((out.len() as u16) + start, &names)?;
              println!("Word: {}", word);
              let [h, l] = to_bytes(word);
              out.push(h);
              out.push(l);
            }
          },
          Directive::Byte(bytes) => {
            for expr in bytes.iter() {
              let byte = expr.resolve_byte((out.len() as u16) + start, &names)?;
              out.push(byte);
            }
          },
          Directive::Define(name, expr) => {
            names.insert(name.clone(), expr.resolve((out.len() as u16) + start, &names)?);
          },
        }
      },
      Token::Label(_) => (), // Nothing to do with these anymore.
      Token::Op(op) => {
        op.resolve(out.len() as u16, &names)?;
        op.assemble(&mut out)?;
      },
    }
  }

  println!("Names: {:?}\n", names);
  println!("Tokens: {:?}\n", tokens);
  println!("Out: {:?}\n", out);

  Ok(out)
}
