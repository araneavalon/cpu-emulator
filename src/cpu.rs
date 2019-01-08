
use std::rc::Rc;

use crate::error::Error;
use crate::clock::Clock;
use crate::bus::Bus;
use crate::addr::{AddrBus, AddrByte};
use crate::connection::Connection;
use crate::mux::Mux;
use crate::input::Input;
use crate::register::{Register, Control as RegisterControl};
use crate::program_counter::{ProgramCounter, Control as PCControl};
use crate::ram::{Ram, Control as RamControl};


#[derive(Debug)]
pub struct Cpu {
	addr: Rc<AddrBus>,
	data: Rc<Bus>,

	i_mux: Rc<Mux>,
	i: Rc<Bus>,
	r: Rc<Bus>,

	pc: Rc<ProgramCounter>,

	a: Rc<Register>,
	b: Rc<Register>,
	x: Rc<Register>,
	y: Rc<Register>,

	ram: Rc<Ram>,
}

impl Cpu {
	pub fn new() -> Cpu {
		let addr = Rc::new(AddrBus::new());
		let data = Rc::new(Bus::new());
		let r = Rc::new(Bus::new());
		let i = Rc::new(Bus::new());

		let pc = Rc::new(ProgramCounter::new(Rc::clone(&data)));
		addr.connect((
			Connection::ProgramCounter(Rc::clone(&pc), PCControl::WriteA(Some(AddrByte::High))),
			Connection::ProgramCounter(Rc::clone(&pc), PCControl::WriteA(Some(AddrByte::Low)))
		));
		data.connect(Connection::ProgramCounter(Rc::clone(&pc), PCControl::WriteD(AddrByte::High)));
		data.connect(Connection::ProgramCounter(Rc::clone(&pc), PCControl::WriteD(AddrByte::Low)));

		let i_mux = Rc::new(Mux::new(vec![Rc::clone(&r), Rc::clone(&data)]));
		i.connect(Connection::Mux(Rc::clone(&i_mux)));

		let a = Rc::new(Register::new(Rc::clone(&i)));
		let b = Rc::new(Register::new(Rc::clone(&i)));
		let x = Rc::new(Register::new(Rc::clone(&i)));
		let y = Rc::new(Register::new(Rc::clone(&i)));
		r.connect(Connection::Register(Rc::clone(&a), RegisterControl::WriteR));
		r.connect(Connection::Register(Rc::clone(&b), RegisterControl::WriteR));
		r.connect(Connection::Register(Rc::clone(&x), RegisterControl::WriteR));
		r.connect(Connection::Register(Rc::clone(&y), RegisterControl::WriteR));
		data.connect(Connection::Register(Rc::clone(&a), RegisterControl::WriteD));
		data.connect(Connection::Register(Rc::clone(&b), RegisterControl::WriteD));
		data.connect(Connection::Register(Rc::clone(&x), RegisterControl::WriteD));
		data.connect(Connection::Register(Rc::clone(&y), RegisterControl::WriteD));

		let ram = Rc::new(Ram::new(Rc::clone(&addr), Rc::clone(&data)));
		data.connect(Connection::Ram(Rc::clone(&ram)));

		Cpu {
			addr: addr,
			data: data,

			pc: pc,

			i_mux: i_mux,
			i: i,
			r: r,

			a: a,
			b: b,
			x: x,
			y: y,

			ram: ram,
		}
	}

	pub fn clock(&self) -> Result<(), Error> {
		self.pc.clock()?;
		self.a.clock()?;
		self.b.clock()?;
		self.x.clock()?;
		self.y.clock()?;
		Ok(())
	}
}
