
pub fn to_bytes(value: u16) -> [u8; 2] {
  [(value >> 8) as u8, value as u8]
}

pub fn from_bytes(value: &[u8; 2]) -> u16 {
  ((value[0] as u16) << 8) | (value[1] as u16)
}

pub fn sign_extend(value: u8) -> u16 {
	if (value & 0x80) != 0 {
		0xFF00 | (value as u16)
	} else {
		value as u16
	}
}
