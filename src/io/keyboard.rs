
use sdl2::keyboard::{
  Keycode,
  Mod,
};


// 0x0000 Keyboard Control/Data
// ........ .......M
//   M    Interrupt for Mod keys.
// SCAGW.EK KKKKKKKK
//   SCAG Shift, Control, Alt, Gui (Windows Key)
//   W    KeyCode is valid.
//   E    KeyCode is "extended" (Non-ascii Character)
//   K    KeyCode


#[derive(Debug)]
pub struct Keyboard {
  mod_interrupt: bool,
  capslock: bool,
  valid: bool,
  key: u16,
}

impl Keyboard {
  pub fn new() -> Keyboard {
    Keyboard {
      mod_interrupt: false,
      capslock: false,
      valid: false,
      key: 0x0000,
    }
  }

  pub fn set_control(&mut self, value: u16) {
    self.mod_interrupt = (value & 0x0001) != 0;
    self.valid = false;
  }

  pub fn get_data(&mut self) -> u16 {
    let key = if self.valid { self.key | 0x0800 } else { self.key & 0xF7FF };
    self.valid = false;
    key
  }

  fn get_mod(keymod: Mod) -> u16 {
    (keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) as u16) << 15 |
      (keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD) as u16) << 14 |
      (keymod.intersects(Mod::LALTMOD | Mod::RALTMOD) as u16)   << 13 |
      (keymod.intersects(Mod::LGUIMOD | Mod::LGUIMOD) as u16)   << 12
  }

  fn get_char(&self, key: Keycode, keymod: Mod) -> Option<u16> {
    let m = self.mod_interrupt;
    let s = self.capslock || keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD);
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

      key if ((key as u16) < 0x80) && s => Some(key.name().as_bytes()[0].to_ascii_uppercase() as u16),
      key if (key as u16) < 0x80 => Some(key.name().as_bytes()[0].to_ascii_lowercase() as u16),

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
      self.capslock = !self.capslock;
    }

    if let Some(chr) = self.get_char(key, keymod) {
      self.key = chr | Keyboard::get_mod(keymod);
      self.valid = true;
      true
    } else {
      self.valid = false;
      false
    }
  }
}
