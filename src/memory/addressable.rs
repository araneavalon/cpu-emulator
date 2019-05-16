
use crate::error::Result;


pub trait Addressable {
  fn name(&self) -> &'static str;
  fn valid(&self, address: u16) -> bool;
  fn read(&self, address: u16) -> Result<u16>;
  fn peek(&self, address: u16) -> Result<u16> { self.read(address) }
  fn write(&mut self, address: u16, value: u16) -> Result<()>;
}
