
use sdl2::keyboard::{
  Keycode,
  Mod,
};

use crate::error::Error;


// Control
// 7......0
//        M
// M  Interrupt for Mod keys.

// Status
// 7......0
// SCAG LCW
// W    Waiting for data to be read.
// C    Data is a "special" key. (ESC, F1-12, BKSP, DEL, RETURN, HOME, UP, DOWN, LEFT, RIGHT, PAUSE, LR Mod Keys)
// L    Data is > 255.
// SCAG Shift, Control, Alt, Gui (Windows)


#[derive(Debug, Eq, PartialEq)]
pub struct Keyboard {
  mod_interrupt: bool,

  key: [u8; 2],
  keymod: Mod,
}

impl Keyboard {
  pub fn new() -> Keyboard {
    Keyboard {
      mod_interrupt: false,

      key: [0x00, 0x00],
      keymod: Mod::NOMOD,
    }
  }

  pub fn get_status(&self, ) -> Result<u8, Error> {
    if self.key == [0x00, 0x00] {
      return Ok(0b00000000)
    }

    let mut status = 0b00000001;

    status |= (self.keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) as u8) << 7;
    status |= (self.keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD) as u8) << 6;
    status |= (self.keymod.intersects(Mod::LALTMOD | Mod::RALTMOD) as u8) << 5;
    status |= (self.keymod.intersects(Mod::LGUIMOD | Mod::LGUIMOD) as u8) << 4;

    if self.key[0] != 0x00 {
      if self.key[0] == 0xFF {
        status |= 0b00000010;
      } else {
        status |= 0b00000100;
      }
    }

    Ok(status)
  }

  pub fn get_data(&self, byte: usize) -> Result<u8, Error> {
    Ok(self.key[byte])
  }

  pub fn set_control(&mut self, value: u8) -> Result<(), Error> {
    self.mod_interrupt = (value & 0x01) != 0;

    Ok(())
  }


  fn get_char(key: Keycode, keymod: Mod) -> Option<u8> {
    match key {
      Keycode::KpPlus => return Some('+' as u8),
      Keycode::Backslash => {
        if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
          return Some('|' as u8)
        }
      }
      _ => (),
    }

    let name = key.name();
    if name.len() == 1 {
      let chr = name.as_bytes()[0];
      if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
        return Some(chr.to_ascii_uppercase())
      } else {
        return Some(chr.to_ascii_lowercase())
      }
    }

    None
  }

  fn get_special(key: Keycode) -> Option<u8> {
    match key {
      Keycode::Escape => Some(0x1B),
      Keycode::Backspace => Some(0x08),
      Keycode::Delete => Some(0x7F),
      Keycode::Tab => Some(0x09),
      Keycode::Return | Keycode::KpEnter => Some(0x0D),

      Keycode::Up    => Some(0x80),
      Keycode::Down  => Some(0x81),
      Keycode::Left  => Some(0x82),
      Keycode::Right => Some(0x83),

      Keycode::Pause => Some(0x88),
      Keycode::Home => Some(0x89),
      Keycode::End => Some(0x8A),

      Keycode::F1  => Some(0xF0),
      Keycode::F2  => Some(0xF1),
      Keycode::F3  => Some(0xF2),
      Keycode::F4  => Some(0xF3),
      Keycode::F5  => Some(0xF4),
      Keycode::F6  => Some(0xF5),
      Keycode::F7  => Some(0xF6),
      Keycode::F8  => Some(0xF7),
      Keycode::F9  => Some(0xF8),
      Keycode::F10 => Some(0xF9),
      Keycode::F11 => Some(0xFA),
      Keycode::F12 => Some(0xFB),

      _ => None,
    }
  }

  fn get_modkey(key: Keycode) -> Option<u8> {
    match key {
      Keycode::LCtrl  => Some(0x90),
      Keycode::RCtrl  => Some(0x91),
      Keycode::LShift => Some(0x92),
      Keycode::RShift => Some(0x93),
      Keycode::LAlt   => Some(0x94),
      Keycode::RAlt   => Some(0x95),
      Keycode::LGui   => Some(0x96),
      Keycode::RGui   => Some(0x97),
      _ => None,
    }
  }

  pub fn pressed(&mut self, key: Keycode, keymod: Mod) -> bool {
    self.keymod = keymod;

    if let Some(chr) = Keyboard::get_modkey(key) {
      if self.mod_interrupt {
        self.key = [0xFF, chr];
      }
      self.mod_interrupt
    } else if let Some(chr) = Keyboard::get_char(key, keymod) {
      self.key = [0x00, chr];
      true
    } else if let Some(chr) = Keyboard::get_special(key) {
      self.key = [0xFF, chr];
      true
    } else {
      // TODO OTHER SPECIAL KEYS
      self.key = [0x00, 0x00];
      false
    }
  }
}
