
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use crate::bus::Bus;
use crate::addr::AddrByte;
use crate::clock::Clock;
use crate::error::Error;


#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Control {
	WriteA(Option<AddrByte>),
	WriteD(AddrByte),
	ReadD(AddrByte),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProgramCounter {
	control: RefCell<HashMap<Control, bool>>,
	h: RefCell<u8>,
	l: RefCell<u8>,
	data: Rc<Bus>,
}

impl ProgramCounter {
	pub fn new(data: Rc<Bus>) -> ProgramCounter {
		ProgramCounter {
			control: RefCell::new(hash_map!{
				Control::WriteA(None) => false,
				Control::ReadD(AddrByte::High) => false,
				Control::ReadD(AddrByte::Low) => false,
				Control::WriteD(AddrByte::High) => false,
				Control::WriteD(AddrByte::Low) => false,
			}),
			h: RefCell::new(0),
			l: RefCell::new(0),
			data: data,
		}
	}

	pub fn set(&self, control: Control, value: bool) {
		let control = match control {
			Control::WriteA(_) => Control::WriteA(None),
			_ => control,
		};
    self.control.borrow_mut().insert(control, value);
	}

	pub fn read(&self, control: &Control) -> Result<Option<u8>, Error> {
		if let Control::WriteA(None) = control {
			return Err(Error::AmbiguousRead(vec![String::from("PC:h"), String::from("PC:l")]))
		}

		let register = match control {
			Control::WriteA(Some(AddrByte::High)) | Control::WriteD(AddrByte::High) => Some(*self.h.borrow()),
			Control::WriteA(Some(AddrByte::Low)) | Control::WriteD(AddrByte::Low) => Some(*self.l.borrow()),
			_ => None,
		};

		let enable = self.control.borrow()[
			match control {
				Control::WriteA(_) => &Control::WriteA(None),
				control => control,
			}
		];

		match (register, enable) {
			(Some(register), true) => Ok(Some(register)),
			(_, _) => Ok(None),
		}
	}
}

impl Clock for ProgramCounter {
	fn clock(&self) -> Result<(), Error> {
		if self.control.borrow()[&Control::ReadD(AddrByte::High)] {
			*self.h.borrow_mut() = self.data.read()?;
		}
		if self.control.borrow()[&Control::ReadD(AddrByte::Low)] {
			*self.l.borrow_mut() = self.data.read()?;
		}
    Ok(())
	}
}

impl fmt::Display for ProgramCounter {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "0x{:X}{:X}", *self.h.borrow(), *self.l.borrow())
	}
}
