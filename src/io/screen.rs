
use std::fmt;
use sdl2::pixels::Color;
use sdl2::render::Canvas;

use crate::math::*;
use crate::error::Error;

// 0x0000-0x07FF Text (2 screens)
// 0x0800-0x0FFF Character (256 8x8)
// 0x1000-0x1EFF Graphics (1 screen)

// 0x004 Cursor Mode EHHH..AB
//         E   Enable
//         HHH Height
//         A   Auto Advance
//         B   Blink
// 0x005 Display Mode FCCCRRMM
//         F   Font size (0=6x8, 1=8x8)
//         CCC Combine Mode (000=OR,001=XOR,011=AND,100=TEXT_ATTRIBUTE)
//         RR  Character Generator (00=ASCII, 01=Japanese Thing, 10=Surprise ;), 11=Character RAM)
//         MM  Display Mode (01=Text, 10=Graphic, 11=Both)
// 0x006 Cursor Address X (0x00-0x27) / (0x00-0x1D)
// 0x007 Cursor Address Y (0x00-0x0F)

const CG_ASCII: &[u8; 2048] = include_bytes!("./cgrom_ascii.hex");
const CG_JAPANESE: &[u8; 2048] = &[0x55; 2048]; // include_bytes!("./cgrom_japanese.hex");
const CG_SURPRISE: &[u8; 2048] = include_bytes!("./cgrom_surprise.hex");

pub struct Screen {
  ram: [u8; 0x1F00],
  display_mode: u8,
  cursor_mode: u8,
  cursor_pos: [u8; 2],
  text_start: u16,
}

impl Screen {
  pub fn new() -> Screen {
    Screen {
      ram: [0x00; 0x1F00],
      display_mode: 0x00,
      cursor_mode: 0x00,
      cursor_pos: [0x00, 0x00],
      text_start: 0x0000,
    }
  }

  pub fn get_display_mode(&self) -> Result<u8, Error> {
    Ok(self.display_mode)
  }
  pub fn set_display_mode(&mut self, value: u8) -> Result<(), Error> {
    self.display_mode = value;
    Ok(())
  }

  pub fn get_cursor_mode(&self) -> Result<u8, Error> {
    Ok(self.cursor_mode)
  }
  pub fn set_cursor_mode(&mut self, value: u8) -> Result<(), Error> {
    self.display_mode = value;
    Ok(())
  }

  pub fn get_cursor_pos(&self, index: usize) -> Result<u8, Error> {
    Ok(self.cursor_pos[index])
  }
  pub fn set_cursor_pos(&mut self, index: usize, value: u8) -> Result<(), Error> {
    self.cursor_pos[index] = value;
    Ok(())
  }

  pub fn get_text_start(&self) -> Result<u16, Error> {
    Ok(self.text_start)
  }
  pub fn set_text_start(&mut self, index: usize, value: u8) -> Result<(), Error> {
    let mut bytes = to_bytes(self.text_start);
    bytes[index] = value;
    self.text_start = from_bytes(&bytes);
    Ok(())
  }

  pub fn get_ram(&self, address: u16) -> Result<u8, Error> {
    Ok(self.ram[address as usize])
  }
  pub fn set_ram(&mut self, address: u16, value: u8) -> Result<(), Error> {
    self.ram[address as usize] = value;
    Ok(())
  }


  fn graphics_size(&self) -> (i32, i32) {
    (240, 128)
  }
  fn text_size(&self) -> (i32, i32) {
    (40, 16) // Is 30 when in 8x8 character mode.
  }
  fn char_size(&self) -> (i32, i32) {
    (6, 8) // Is 8x8 when in 8x8 character mode. (Duh.)
  }

  pub fn draw<T: sdl2::render::RenderTarget>(&self, canvas: &mut Canvas<T>, bg: Color, fg: Color) -> Result<(), Error> {
    // ASCII Text mode only for now.

    canvas.set_draw_color(fg);
    canvas.draw_point((0, 0)).unwrap();

    let (char_w, char_h) = self.char_size();
    let (columns, rows) = self.text_size();
    for row in 0..rows {
      for col in 0..columns {
        let character = self.ram[(row * columns + col) as usize];
        for y in 0..char_h {
          let line = CG_ASCII[((character as i32) * char_h + y) as usize];
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

impl PartialEq for Screen {
  fn eq(&self, other: &Screen) -> bool {
    self.ram[..] == other.ram[..] &&
      self.display_mode == other.display_mode &&
      self.cursor_mode == other.cursor_mode &&
      self.cursor_pos == other.cursor_pos &&
      self.text_start == other.text_start
  }

  fn ne(&self, other: &Screen) -> bool {
    self.ram[..] != other.ram[..] ||
      self.display_mode != other.display_mode ||
      self.cursor_mode != other.cursor_mode ||
      self.cursor_pos != other.cursor_pos ||
      self.text_start != other.text_start
  }
}

impl Eq for Screen {}

impl fmt::Debug for Screen {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Screen.")
  }
}
