
// Video Ram
// 0x0000-0x07FF Text (2 screens)
// 0x0800-0x0FFF Character (256 8x8)
// 0x1000-0x1EFF Graphics (1 screen)
// 0x1F00-0x1FFF IO Ports

// IO Ports
// 0x000 Bank Register
// 0x001 Keyboard Control/Status
// 0x002 Keyboard Data 1
// 0x003 Keyboard Data 0
// 0x004 Cursor Mode
// 0x005 Display Mode
// 0x006 Cursor Address X (0x00-0x27) / (0x00-0x1D)
// 0x007 Cursor Address Y (0x00-0x0F)
// 0x008 Text Start Address (0x000-0x3FF)
// SD Stuff


mod keyboard;
mod screen;

use crate::math::*;
use crate::error::Error;
use crate::io::keyboard::Keyboard;
use crate::io::screen::Screen;


// SCREEN:
// https://www.mouser.com/datasheet/2/291/NHD-240128WG-ATFH-VZ-27453.pdf
// http://www.newhavendisplay.com/app_notes/RA6963.pdf
// LEVEL SHIFTER:
// http://www.ti.com/lit/ds/symlink/txb0108.pdf
// IO CONTROLLER:
// https://www.st.com/resource/en/datasheet/stm32f072c8.pdf


#[derive(Debug, PartialEq, Eq)]
pub struct Io {
  pub keyboard: Keyboard,
  pub screen: Screen,
}

impl Io {
  pub fn new() -> Io {
    Io {
      keyboard: Keyboard::new(),
      screen: Screen::new(),
    }
  }

  pub fn get_value(&self, address: u16) -> Result<u8, Error> {
    if address < 0x1F00 {
      return Ok(self.screen.get_ram(address)?)
    }
    match address & 0x00FF {
      0x01 => Ok(self.keyboard.get_status()?),
      0x02 => Ok(self.keyboard.get_data(1)?),
      0x03 => Ok(self.keyboard.get_data(0)?),
      0x04 => Ok(self.screen.get_display_mode()?),
      0x05 => Ok(self.screen.get_cursor_mode()?),
      0x06 => Ok(self.screen.get_cursor_pos(0)?),
      0x07 => Ok(self.screen.get_cursor_pos(1)?),
      0x08 => Ok(to_bytes(self.screen.get_text_start()?)[1]),
      0x09 => Ok(to_bytes(self.screen.get_text_start()?)[0]),
      _ => Err(Error::Error(String::from("Unknown IO address."))),
    }
  }

  pub fn set_value(&mut self, address: u16, value: u8) -> Result<(), Error> {
    if address <= 0x1F00 {
      return Ok(self.screen.set_ram(address, value)?)
    }
    match address & 0x00FF {
      0x01 => Ok(self.keyboard.set_control(value)?),
      0x02 => Err(Error::InvalidWrite(String::from("IO address 0x02 is not writable."))),
      0x03 => Err(Error::InvalidWrite(String::from("IO address 0x03 is not writable."))),
      0x04 => Ok(self.screen.set_display_mode(value)?),
      0x05 => Ok(self.screen.set_cursor_mode(value)?),
      0x06 => Ok(self.screen.set_cursor_pos(0, value)?),
      0x07 => Ok(self.screen.set_cursor_pos(1, value)?),
      0x08 => Ok(self.screen.set_text_start(1, value)?),
      0x09 => Ok(self.screen.set_text_start(0, value)?),
      _ => Err(Error::Error(String::from("Unknown IO address."))),
    }
  }
}
