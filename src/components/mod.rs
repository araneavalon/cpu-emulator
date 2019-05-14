
use crate::control::Control;
use crate::error::Result;

mod address_register;
mod alu;
mod flags;
mod instruction_register;
mod link_register;
mod program_counter;
mod register_file;
mod stack_pointers;

pub use address_register::AddressRegister;
pub use alu::Alu;
pub use flags::{Flag, Flags};
pub use instruction_register::InstructionRegister;
pub use link_register::LinkRegister;
pub use program_counter::ProgramCounter;
pub use register_file::RegisterFile;
pub use stack_pointers::StackPointers;


pub trait BusComponent {
  fn name(&self) -> &'static str;
  fn set_control(&mut self, control: Control);
  fn load(&mut self, value: u16) -> Result<()>;
  fn data(&self) -> Result<Option<u16>> { Ok(None) }
  fn address(&self) -> Result<Option<u16>> { Ok(None) }
}
