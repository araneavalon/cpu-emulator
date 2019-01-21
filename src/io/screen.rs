
use crate::math::*;

// 0x004 Cursor Mode .HHH..AB
//       	 HHH Height
//         A   Advance
//         B   Blink
// 0x005 Display Mode FFRCCCMM
//         F   Font size (0=6x8, 1=8x8)
//         F   Internal Font Select
//         R   Character ROM/RAM
//         CCC Combine Mode (OR,XOR,,AND,TEXT_ATTRIBUTE)
//         MM  Display Mode (01=Text, 10=Graphic, 11=Both)
// 0x006 Cursor Address X (0x00-0x27) / (0x00-0x1D)
// 0x007 Cursor Address Y (0x00-0x0F)

const FONT_0: [u8; 128 * 8] = include_bytes!("./cgrom_0.hex");
const FONT_1: [u8; 128 * 8] = include_bytes!("./cgrom_1.hex");

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
		let mut bytes = to_bytes(self.text_pos);
		bytes[index] = value;
		self.text_pos = from_bytes(&bytes);
	}

	pub fn get_ram(&self, address: u16) -> Result<u8, Error> {
		Ok(self.ram[address as usize])
	}
	pub fn set_ram(&mut self, address: u16, value: u8) -> Result<(), Error> {
		self.ram[address as usize] = value;
		Ok(())
	}


	fn terminal_control(&self) {

	}
}
