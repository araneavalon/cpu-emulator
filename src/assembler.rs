
use std::collections::HashMap;

mod error;
mod tokens;
mod parser;
mod assembler;

use self::error::Error;
use self::tokens::Token;
use self::tokens::Op;


pub fn assemble(input: &str) -> Result<Vec<u8>, Error> {
  let sections: Vec<(u16, Vec<Token>)> = parser::parse(input)?;

  println!("\nSections: {:?}\n", sections);

  let mut labels: HashMap<String, u16> = HashMap::new();

  let mut sections: Vec<(u16, Vec<Op>)> = sections.into_iter().map(|(start, section)| {
    let mut address = start;
    (start, section.into_iter().filter_map(|token| {
      match token {
        Token::Label(label) => {
          labels.insert(label.clone(), address);
          None
        },
        Token::Op(op) => {
          address += op.len();
          Some(op)
        },
      }
    }).collect())
  }).collect();

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

    for op in section.iter_mut() {
      op.resolve(&labels)?;
      op.assemble(&mut out)?;
    }
  }

  println!("Sections: {:?}\n", sections);
  println!("Out: {:?}\n", out);

  Ok(out)
}
