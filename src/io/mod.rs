
use std::ops::{Index, IndexMut};
use std::fmt;

mod keyboard;
mod screen;

use crate::error::Result;

pub use screen::Screen;


// 0xC000 0xCBFF   Text (3 screens) (only uses low byte)
// 0xCC00 0xCFFF   Character (256 8x8)
// 0xD000 0xDDFF   Graphics (2 screens)
// 0xDE00 0xDFFF   IO Ports (512)

// 0xDE00 0xDE03   SCREEN
// 0xDE04          KEYBOARD

// SD Card Stuff?

pub struct Io {
  screen: Screen,
  // keyboard: Keyboard,
  io: [u16; 0x0200],
}

impl Io {
  pub fn new() -> Io {
    Io {
      screen: Screen::new(),
      io: [0x0000; 0x0200],
    }
  }

  pub fn name(&self) -> &'static str {
    " IO"
  }

  pub fn valid(&self, address: u16) -> bool {
    (0xC000 <= address) && (address < 0xE000)
  }

  pub fn screen(&self) -> Result<&Screen> {
    Ok(&self.screen)
  }
}

impl Index<u16> for Io {
  type Output = u16;

  fn index(&self, address: u16) -> &u16 {
    if self.screen.valid(address) {
      &self.screen[address]
    } else {
      panic!("Invalid address used to index Io. Check Io::valid.");
    }
  }
}

impl IndexMut<u16> for Io {
  fn index_mut(&mut self, address: u16) -> &mut u16 {
    if self.screen.valid(address) {
      &mut self.screen[address]
    } else {
      panic!("Invalid address used to index Io. Check Io::valid.");
    }
  }
}

impl fmt::Debug for Io {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Io {{ screen: {:?}, io: {:?} }}",
      self.screen, &self.io[..])
  }
}
