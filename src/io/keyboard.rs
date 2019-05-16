
use std::cell::RefCell;
use sdl2::keyboard::{
  Keycode,
  Mod,
};

use crate::memory::Addressable;
use crate::error::{
  Result,
  Error,
};


// 0xDE04 Keyboard Control/Data
// ........ ......CM
//   M    Interrupt for Mod keys.
//   C    Capslock enabled.
// SCAGW.EK KKKKKKKK
//   SCAG Shift, Control, Alt, Gui (Windows Key)
//   W    KeyCode is valid.
//   E    KeyCode is "extended" (Non-ascii Character)
//   K    KeyCode


#[derive(Debug)]
pub struct Keyboard {
  mode: u16,
  keys: RefCell<Vec<u16>>,
}

impl Keyboard {
  pub fn new() -> Keyboard {
    Keyboard {
      mode: 0x0000,
      keys: RefCell::new(Vec::new()),
    }
  }

  fn get_mod(keymod: Mod) -> u16 {
    (keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) as u16) << 15 |
      (keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD) as u16) << 14 |
      (keymod.intersects(Mod::LALTMOD | Mod::RALTMOD) as u16)   << 13 |
      (keymod.intersects(Mod::LGUIMOD | Mod::LGUIMOD) as u16)   << 12
  }

  fn get_char(&self, key: Keycode, keymod: Mod) -> Option<u16> {
    let m = (self.mode & 0x0001) != 0;
    let c = (self.mode & 0x0002) != 0;
    let s = keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD);
    match key {
      Keycode::Backspace      => Some(0x0008),
      Keycode::Tab            => Some(0x0009),
      Keycode::Return |
      Keycode::KpEnter        => Some(0x000D),
      Keycode::Escape         => Some(0x001B),
      Keycode::Delete         => Some(0x007F),

      Keycode::KpDivide       => Some('/' as u16),
      Keycode::KpMultiply     => Some('*' as u16),
      Keycode::KpMinus        => Some('-' as u16),
      Keycode::KpPlus         => Some('+' as u16),
      Keycode::Kp1            => Some('1' as u16),
      Keycode::Kp2            => Some('2' as u16),
      Keycode::Kp3            => Some('3' as u16),
      Keycode::Kp4            => Some('4' as u16),
      Keycode::Kp5            => Some('5' as u16),
      Keycode::Kp6            => Some('6' as u16),
      Keycode::Kp7            => Some('7' as u16),
      Keycode::Kp8            => Some('8' as u16),
      Keycode::Kp9            => Some('+' as u16),
      Keycode::Kp0            => Some('0' as u16),
      Keycode::KpPeriod       => Some('.' as u16),
      Keycode::KpEquals       => Some('=' as u16),
      Keycode::KpComma        => Some(',' as u16),

      Keycode::Backslash if s => Some('|' as u16),

      key if ((key as u16) < 0x80) && (key.name().len() == 1) && (s ^ c) =>
        Some(key.name().as_bytes()[0].to_ascii_uppercase() as u16),
      key if ((key as u16) < 0x80) && (key.name().len() == 1) && !(s ^ c) =>
        Some(key.name().as_bytes()[0].to_ascii_lowercase() as u16),

      Keycode::CapsLock  if m => Some(0x0239),

      Keycode::F1             => Some(0x023A),
      Keycode::F2             => Some(0x023B),
      Keycode::F3             => Some(0x023C),
      Keycode::F4             => Some(0x023D),
      Keycode::F5             => Some(0x023E),
      Keycode::F6             => Some(0x023F),
      Keycode::F7             => Some(0x0240),
      Keycode::F8             => Some(0x0241),
      Keycode::F9             => Some(0x0242),
      Keycode::F10            => Some(0x0243),
      Keycode::F11            => Some(0x0244),
      Keycode::F12            => Some(0x0245),

      Keycode::Home           => Some(0x024A),
      Keycode::PageUp         => Some(0x024B),
      Keycode::End            => Some(0x024D),
      Keycode::PageDown       => Some(0x024E),

      Keycode::Right          => Some(0x024F),
      Keycode::Left           => Some(0x0250),
      Keycode::Down           => Some(0x0251),
      Keycode::Up             => Some(0x0252),

      Keycode::LCtrl     if m => Some(0x02E0),
      Keycode::LShift    if m => Some(0x02E1),
      Keycode::LAlt      if m => Some(0x02E2),
      Keycode::LGui      if m => Some(0x02E3),
      Keycode::RCtrl     if m => Some(0x02E4),
      Keycode::RShift    if m => Some(0x02E5),
      Keycode::RAlt      if m => Some(0x02E6),
      Keycode::RGui      if m => Some(0x02E7),

      _ => None,
    }
  }

  pub fn pressed(&mut self, key: Keycode, keymod: Mod) -> bool {
    if key == Keycode::CapsLock {
      self.mode = self.mode ^ 0x0002;
    }

    if let Some(chr) = self.get_char(key, keymod) {
      self.keys.borrow_mut().push(chr | Keyboard::get_mod(keymod) | 0x0800);
    }

    self.keys.borrow().len() > 0
  }
}

impl Addressable for Keyboard {
  fn name(&self) -> &'static str {
    "Keyboard"
  }

  fn valid(&self, address: u16) -> bool {
    address == 0xDE04
  }

  fn read(&self, address: u16) -> Result<u16> {
    match address {
      0xDE04 if self.keys.borrow().len() > 0 => Ok(self.keys.borrow_mut().remove(0)),
      0xDE04 => Ok(0x0000),
      _ => Err(Error::InvalidRead(address, "Invalid read from Keyboard.")),
    }
  }

  fn peek(&self, address: u16) -> Result<u16> {
    match address {
      0xDE04 if self.keys.borrow().len() > 0 => Ok(self.keys.borrow()[0]),
      0xDE04 => Ok(0x0000),
      _ => Err(Error::InvalidRead(address, "Invalid peek from Keyboard.")),
    }
  }

  fn write(&mut self, address: u16, value: u16) -> Result<()> {
    match address {
      0xDE04 => self.mode = value,
      _ => return Err(Error::InvalidWrite(address, "Invalid write to Keyboard.")),
    }
    Ok(())
  }
}
