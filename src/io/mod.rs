
use std::fmt;

mod keyboard;
mod screen;

use crate::memory::Addressable;
use crate::error::{
  Result,
  Error,
};

pub use screen::Screen;
pub use keyboard::Keyboard;


// 0xC000 0xCBFF   Text (3 screens) (only uses low byte)
// 0xCC00 0xCFFF   Character (256 8x8)
// 0xD000 0xDDFF   Graphics (2 screens)
// 0xDE00 0xDFFF   IO Ports (512)

// 0xDE00 0xDE03   SCREEN
// 0xDE04          KEYBOARD

// SD Card Stuff?

pub struct Io {
  screen: Screen,
  keyboard: Keyboard,
  io: [u16; 0x0200],
}

impl Io {
  pub fn new() -> Io {
    Io {
      screen: Screen::new(),
      keyboard: Keyboard::new(),
      io: [0x0000; 0x0200],
    }
  }

  pub fn screen(&self) -> Result<&Screen> {
    Ok(&self.screen)
  }

  pub fn keyboard(&mut self) -> Result<&mut Keyboard> {
    Ok(&mut self.keyboard)
  }
}

impl Addressable for Io {
  fn name(&self) -> &'static str {
    " IO"
  }

  fn valid(&self, address: u16) -> bool {
    (0xC000 <= address) && (address < 0xE000)
  }

  fn read(&self, address: u16) -> Result<u16> {
    if self.screen.valid(address) {
      self.screen.read(address)
    } else if self.keyboard.valid(address) {
      self.keyboard.read(address)
    } else {
      Err(Error::InvalidRead(address, "Could not read from IO RAM."))
    }
  }

  fn peek(&self, address: u16) -> Result<u16> {
    if self.screen.valid(address) {
      self.screen.peek(address)
    } else if self.keyboard.valid(address) {
      self.keyboard.peek(address)
    } else {
      Err(Error::InvalidRead(address, "Could not peek from IO RAM."))
    }
  }

  fn write(&mut self, address: u16, value: u16) -> Result<()> {
    if self.screen.valid(address) {
      self.screen.write(address, value)
    } else if self.keyboard.valid(address) {
      self.keyboard.write(address, value)
    } else {
      Err(Error::InvalidWrite(address, "Could not write to IO RAM."))
    }
  }
}

impl fmt::Debug for Io {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Io {{ screen: {:?}, keyboard: {:?}, io: {{:?}} }}",
      self.screen, self.keyboard, /*&self.io[..]*/)
  }
}
