
use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::path::Path;

use super::error::{
  Result,
  Error,
};


fn read_file<P: AsRef<Path>>(path: &P) -> io::Result<String> {
  let mut buffer = String::new();
  File::open(path)?.read_to_string(&mut buffer)?;
  Ok(buffer)
}

fn process_file<P: AsRef<Path>>(path: P, buffer: &mut String) -> Result<()> {
  let file = match read_file(&path) {
    Err(error) => return Err(Error::file(String::from(path.as_ref().to_string_lossy()), error)),
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
        return Err(Error::invalid_filename(String::from(path.as_ref().to_string_lossy()), String::from(""))),
      (_, '\n') if import => {
        match path.as_ref().parent() {
          None =>
            return Err(Error::invalid_filename(String::from(path.as_ref().to_string_lossy()), String::from(""))),
          Some(parent) => {
            process_file(parent.join(&file[start..index]), buffer)?;
            import = false;
          },
        }
      },
      _ if import => (),
      _ => buffer.push(c),
    }
    previous = Some(c);
  }

  if import {
    Err(Error::invalid_filename(String::from(path.as_ref().to_string_lossy()), String::from(&file[start..])))
  } else {
    Ok(())
  }
}

pub fn preprocess(root: &str) -> Result<String> {
  let mut buffer = String::new();
  process_file(root, &mut buffer)?;
  Ok(buffer)
}
