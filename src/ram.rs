
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use crate::bus::Bus;
use crate::addr::AddrBus;
use crate::clock::Clock;
use crate::error::Error;


const MEM_SIZE: usize = 0x1FFFF;


#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Control {
	ReadWrite,
	Enable,
}


pub struct Ram {
	control: RefCell<HashMap<Control, bool>>,
	memory: RefCell<[u8; MEM_SIZE]>, // TODO BANK SWITCHING IN HERE YO
	addr: Rc<AddrBus>,
	data: Rc<Bus>,
}

impl Ram {
	pub fn new(addr: Rc<AddrBus>, data: Rc<Bus>) -> Ram {
		Ram {
			control: RefCell::new(hash_map!{
				Control::ReadWrite => false,
				Control::Enable => false,
			}),
			memory: RefCell::new([0; MEM_SIZE]), // TODO BANK SWITCHING
			addr: addr,
			data: data,
		}
	}

	fn address(&self) -> Result<usize, Error> {
		let (h, l) = self.addr.read()?;
		Ok((h as usize) << 8 | (l as usize))
	}

	pub fn read(&self) -> Result<Option<u8>, Error> {
		let control = self.control.borrow();
		if control[&Control::Enable] && !control[&Control::ReadWrite] {
			Ok(Some(self.memory.borrow()[self.address()?]))
		} else {
			Ok(None)
		}
	}
}

impl Clock for Ram {
	fn clock(&self) -> Result<(), Error> {
		let control = self.control.borrow();
		if control[&Control::Enable] && control[&Control::ReadWrite] {
			self.memory.borrow_mut()[self.address()?] = self.data.read()?;
		}
		Ok(())
	}
}

impl fmt::Debug for Ram {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{{ control: {:?}, addr: {:?}, data: {:?}, memory: [u8; {:#X}] }}", self.control, self.addr, self.data, MEM_SIZE)
	}
}

impl PartialEq for Ram {
	fn eq(&self, other: &Ram) -> bool {
		self.control == other.control &&
			self.memory.borrow()[..] == other.memory.borrow()[..] &&
			self.addr == other.addr &&
			self.data == other.data
	}

	fn ne(&self, other: &Ram) -> bool {
		self.control != other.control ||
			self.memory.borrow()[..] != other.memory.borrow()[..] ||
			self.addr != other.addr ||
			self.data != other.data
	}
}

impl Eq for Ram {}
