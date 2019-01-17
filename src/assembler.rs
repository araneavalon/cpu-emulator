
use std::collections::HashMap;

mod error;
mod tokens;
mod parser;
mod assembler;

use self::error::Error;
use self::tokens::Token;


pub fn assemble(input: &str) -> Result<Vec<u8>, Error> {
  let mut sections: Vec<(u16, Vec<Token>)> = parser::parse(input)?;

  let mut labels: HashMap<String, u16> = HashMap::new();

  for (start, section) in sections.iter() {
    let mut address = *start;
    for token in section.iter() {
      match token {
        Token::Label(label) => { labels.insert(label.clone(), address); },
        Token::Op(op) => address += op.len(),
      }
    }
  }

  println!("Sections: {:?}\n", sections);
  println!("Labels: {:?}\n", labels);

  let mut out: Vec<u8> = Vec::new();

  let first_section = sections[0].0;
  for (start, section) in sections.iter_mut() {
    let len = out.len() as u16;

    if (*start - first_section) < len {
      return Err(Error::SectionOverlap(len + first_section, *start));
    }
    for _i in 0..(*start - first_section - len) {
      out.push(0x00);
    }

    for token in section.iter_mut() {
      match token {
        Token::Label(_) => (),
        Token::Op(op) => {
          op.resolve(&labels)?;
          op.assemble(&mut out)?;
        },
      }
    }
  }

  println!("Sections: {:?}\n", sections);

  Ok(out)
}
