
use std::io::prelude::*;
use std::io;
use std::fs::File;

use super::error::{
  Result,
  Error,
};


fn read_file(file: &str) -> io::Result<String> {
  let mut buffer = String::new();
  File::open(file)?.read_to_string(&mut buffer)?;
  Ok(buffer)
}

fn process_file(path: &str, buffer: &mut String) -> Result<()> {
  let file = match read_file(path) {
    Err(error) => return Err(Error::file(String::from(path), error)),
    Ok(file) => file,
  };

  let mut previous = None;
  let mut import = false;
  let mut start = 0;

  let mut iter = file.char_indices().peekable();
  while let Some((index, c)) = iter.next() {
    match (previous, c) {
      (None, '@') | (Some('\n'), '@') if !import => {
        import = true;
        match iter.peek() {
          Some((index, _)) => start = *index,
          None => start = index, // So that error message has the right thing.
        }
      },
      (_, '\n') if import && start == index =>
        return Err(Error::invalid_filename(String::from(path), String::from(""))),
      (_, '\n') if import => {
        process_file(&file[start..index], buffer)?;
        import = false;
      },
      _ if import => (),
      _ => buffer.push(c),
    }
    previous = Some(c);
  }

  if import {
    Err(Error::invalid_filename(String::from(path), String::from(&file[start..])))
  } else {
    Ok(())
  }
}

pub fn preprocess(root: &str) -> Result<String> {
  let mut buffer = String::new();
  process_file(root, &mut buffer)?;
  Ok(buffer)
}
