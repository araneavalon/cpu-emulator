
use crate::control::Control;

pub mod address_register;
pub mod alu;
pub mod flags;
pub mod instruction_register;
pub mod link_register;
pub mod memory;
pub mod program_counter;
pub mod register_file;
pub mod stack_pointers;


pub trait BusComponent {
  fn set_control(&mut self, control: Control);
  fn load(&mut self, value: u16);
  fn data(&self) -> Option<u16> { None }
  fn address(&self) -> Option<u16> { None }
}
