
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use std::fmt;

use crate::memory::Addressable;
use crate::error::{
  Result,
  Error,
};

// 0xC000 0xCBFF   Text (3 screens) (only uses low byte)
// 0xCC00 0xCFFF   Character (256 8x8)
// 0xD000 0xDDFF   Graphics (2 screens)

// 0x0000 Screen Mode
// EHHH..AB FCCCRRMM
// Cursor:
//   E    Enable
//   HHH  Height
//   A    Auto Advance
//   B    Blink
// Display:
//   F    Font Size (0=6x8, 1=8x8)
//   CCC  Combine Mode (000=OR, 001=XOR, 011=AND, 100=TEXT_ATTRIBUTE)
//   RR   Character Generator (00=ASCII, 01=Japanese, 10=Surprise, 11=Character RAM)
//   MM   Mode (01=Text, 10=Graphic, 11=Both)
// 0x0000 Cursor Address X (0x0000-0x0027) / (0x0000-0x001D)
// 0x0000 Cursor Address Y (0x0000-0x000F)
// 0x0000 Text Start Address (0x0000-0x07FF)


const CG_ASCII: &[u8; 2048] = include_bytes!("./cgrom_ascii.hex");
const CG_JAPANESE: &[u8; 2048] = &[0x55; 2048]; // include_bytes!("./cgrom_japanese.hex");
const CG_SURPRISE: &[u8; 2048] = include_bytes!("./cgrom_surprise.hex");

const RAM_SIZE: usize   = 0x1E00;
const RAM_OFFSET: usize = 0xC000;

#[derive(Debug)]
enum CharSet<'a> {
  Ascii,
  Japanese,
  Surprise,
  Ram(&'a [u16]),
}
impl<'a> CharSet<'a> {
  fn get(&self, index: usize) -> u8 {
    match self {
      CharSet::Ascii => CG_ASCII[index],
      CharSet::Japanese => CG_JAPANESE[index],
      CharSet::Surprise => CG_SURPRISE[index],
      CharSet::Ram(slice) => {
        if (index & 1) == 0 {
          (slice[index >> 1] & 0x00FF) as u8
        } else {
          ((slice[index >> 1] & 0xFF00) >> 8) as u8
        }
      },
    }
  }
}

pub struct Screen {
  data: [u16; RAM_SIZE],
  mode: u16,
  cursor_pos: [u16; 2],
  text_start: u16,
}

impl Screen {
  pub fn new() -> Screen {
    Screen {
      data: [0x0000; RAM_SIZE],
      mode: 0x0000,
      cursor_pos: [0x0000, 0x0000],
      text_start: 0x0000,
    }
  }

  fn chars(&self) -> CharSet {
    match (self.mode & 0x000C) >> 2 {
      0 => CharSet::Ascii,
      1 => CharSet::Japanese,
      2 => CharSet::Surprise,
      3 => CharSet::Ram(&self.data[0xCC00..=0xCFFF]),
      _ => panic!("Literally impossible."), // TODO error
    }
  }
  fn graphics_size(&self) -> (i32, i32) {
    (240, 128)
  }
  fn text_size(&self) -> (i32, i32) {
    if (self.mode & 0x0080) != 0 {
      (30, 16) // 8x8
    } else {
      (40, 16) // 6x8
    }
  }
  fn char_size(&self) -> (i32, i32) {
    if (self.mode & 0x0080) != 0 {
      (8, 8)
    } else {
      (6, 8)
    }
  }

  pub fn draw<T: sdl2::render::RenderTarget>(&self, canvas: &mut Canvas<T>, bg: Color, fg: Color) -> Result<()> {
    canvas.set_draw_color(fg);

    let chars = self.chars();
    let (char_w, char_h) = self.char_size();
    let (columns, rows) = self.text_size();
    for row in 0..rows {
      for col in 0..columns {
        let character = (self.data[(row * columns + col) as usize] & 0x00FF) as i32;
        for y in 0..char_h {
          let line = chars.get((character * char_h + y) as usize); // TODO do bounds checking
          for x in 0..char_w {
            if ((line >> (char_w - x - 1)) & 1) != 0 {
              canvas.draw_point(((col * char_w) + x, (row * char_h) + y)).unwrap(); // TODO ERROR
            }
          }
        }
      }
    }

    Ok(())
  }
}

impl Addressable for Screen {
  fn name(&self) -> &'static str {
    "Screen"
  }

  fn valid(&self, address: u16) -> bool {
    match address {
      0xC000 ... 0xDDFF |
      0xDE00 ... 0xDE03 => true,
      _ => false,
    }
  }

  fn read(&self, address: u16) -> Result<u16> {
    match address {
      0xC000 ... 0xDDFF => Ok(self.data[(address as usize) - RAM_OFFSET]),
      0xDE00 => Ok(self.mode),
      0xDE01 => Ok(self.cursor_pos[0]),
      0xDE02 => Ok(self.cursor_pos[1]),
      0xDE03 => Ok(self.text_start),
      _ => Err(Error::InvalidRead(address, "Invalid read from Screen RAM.")),
    }
  }

  fn write(&mut self, address: u16, value: u16) -> Result<()> {
    match address {
      0xC000 ... 0xDDFF => self.data[(address as usize) - RAM_OFFSET] = value,
      0xDE00 => self.mode = value,
      0xDE01 => self.cursor_pos[0] = value,
      0xDE02 => self.cursor_pos[1] = value,
      0xDE03 => self.text_start = value,
      _ => return Err(Error::InvalidWrite(address, "Invalid write to Screen RAM.")),
    }
    Ok(())
  }
}

impl fmt::Debug for Screen {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Screen {{ data: vec![], mode: 0x{:04X}, cursor_pos: [0x{:04X}, 0x{:04X}], text_start: 0x{:04X} }}",
      self.mode, self.cursor_pos[0], self.cursor_pos[1], self.text_start)
  }
}
