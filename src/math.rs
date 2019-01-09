
pub fn to_bytes(value: u16) -> [u8; 2] {
  [(value >> 8) as u8, value as u8]
}

pub fn from_bytes(value: &[u8; 2]) -> u16 {
  ((value[0] << 8) as u16) | (value[1] as u16)
}
