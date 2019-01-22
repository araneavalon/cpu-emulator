
use framebuffer::{Framebuffer, KdMode};
use std::io::{self, Read};
use std::fmt;

use crate::bus;
use crate::error::Error;

use crate::bus::Device;

use crate::instructions;
use crate::control_logic::ControlLogic;

use crate::program_counter::ProgramCounter;
use crate::stack_pointer::StackPointer;
use crate::register::Register;
use crate::address_register::AddressRegister;
use crate::memory_controller::MemoryController;
use crate::flags_register::FlagsRegister;
use crate::alu::Alu;


pub struct Cpu<T: instructions::Set> {
  halt: bool,

  ir: ControlLogic<T>,
  pc: ProgramCounter,
  sp: StackPointer,

  a: Register,
  b: Register,
  x: Register,
  y: Register,

  address: AddressRegister,

  memory: MemoryController,

  flags: FlagsRegister,
  alu: Alu,
}

impl<T: instructions::Set> Cpu<T> {
  pub fn new(instructions: Box<T>) -> Cpu<T> {
    Cpu {
      halt: false,

      ir: ControlLogic::new(instructions),
      pc: ProgramCounter::new(),
      sp: StackPointer::new(),

      a: Register::new(),
      b: Register::new(),
      x: Register::new(),
      y: Register::new(),

      address: AddressRegister::new(),

      memory: MemoryController::new(),

      flags: FlagsRegister::new(),
      alu: Alu::new(),
    }
  }

  fn update(&mut self) -> Result<bool, Error> {
    let control = self.ir.get_control(self.flags.get_flags());

    self.ir.update(control.Instruction)?;
    self.pc.update(control.ProgramCounter)?;
    self.sp.update(control.StackPointer)?;

    self.a.update(control.A)?;
    self.b.update(control.B)?;
    self.x.update(control.X)?;
    self.y.update(control.Y)?;

    self.address.update(control.AddressRegister)?;

    self.memory.update(control.Memory)?;

    self.flags.update(control.FlagsRegister)?;
    self.alu.update(control.Alu)?;

    Ok(self.ir.halt())
  }

  fn merge_states(&self, states: Vec<bus::State>) -> Result<bus::State, Error> {
    let mut state = bus::State { data: None, addr: None };
    for s in states.iter() {
      match (s.data, state.data) {
        (Some(_), Some(_)) => return Err(Error::BusConflict(vec![String::from("data")])),
        (Some(_), None)    => state.data = s.data,
        (None, _)          => (),
      }
      match (s.addr, state.addr) {
        (Some(_), Some(_)) => return Err(Error::BusConflict(vec![String::from("addr")])),
        (Some(_), None)    => state.addr = s.addr,
        (None, _)          => (),
      }
    }
    Ok(state)
  }

  fn bus_state(&mut self) -> Result<bus::State, Error> {
    let state = self.merge_states(vec![
      self.ir.read()?,
      self.pc.read()?,
      self.sp.read()?,
      self.address.read()?,
      self.alu.read()?,
    ])?;

    self.memory.set_addr(&state)?;

    let state = self.merge_states(vec![
      state,
      self.flags.read()?,
      self.a.read()?,
      self.b.read()?,
      self.x.read()?,
      self.y.read()?,
      self.memory.read()?,
    ])?;

    Ok(state)
  }

  fn clk(&mut self, state: &bus::State) -> Result<(), Error> {
    self.ir.clk(state)?;
    self.pc.clk(state)?;
    self.sp.clk(state)?;

    self.a.clk(state)?;
    self.b.clk(state)?;
    self.x.clk(state)?;
    self.y.clk(state)?;

    self.address.clk(state)?;

    self.memory.clk(state)?;

    self.flags.clk(state)?;
    self.alu.clk(state)?;

    self.flags.set_flags(self.alu.get_flags());

    Ok(())
  }


  fn tick(&mut self, ns: std::time::Duration) -> Result<bool, Error> {
    let halt = self.update()?;
    std::thread::sleep(ns);

    let state = self.bus_state()?;
    self.clk(&state)?;
    std::thread::sleep(ns);

    Ok(halt)
  }

  fn draw(&self, fb: &mut Framebuffer, scale: u32) -> Result<(), Error> {
    let frame: Vec<u8> = self.memory.io.screen.get_frame()?;
    fb.write_frame(&frame);
    Ok(())
  }

  fn setup_fb(&self, fb: &mut Framebuffer) -> u32 {
    // let w = fb.var_screen_info.xres;
    // let h = fb.var_screen_info.yres;
    // let scale = std::cmp::min(w / 240, h / 128);
    // let xo = (w - (240 * scale)) / 2;
    // let yo = (h - (128 * scale)) / 2;

    let info = &mut fb.var_screen_info;

    info.bits_per_pixel = 1;
    info.grayscale = 1;

    Framebuffer::put_var_screeninfo(&fb.device, &info).unwrap();

    0
  }

  pub fn run(&mut self, hz: u64) -> Result<(), Error> {
    let ticks_per_frame = hz / 60;
    let tick = std::time::Duration::from_nanos(1_000_000_000 / hz);
    let half_tick = std::time::Duration::from_nanos(500_000_000 / 2);

    let stdin = io::stdin();

    let mut fb = Framebuffer::new("/dev/fb0").unwrap();
    let scale = self.setup_fb(&mut fb);
    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();

    let mut ticks: u64 = 0;
    loop {
      self.halt = self.tick(half_tick)?;

      ticks += 1;
      if ticks >= ticks_per_frame {
        ticks = 0;
        self.draw(&mut fb, scale)?;
      }

      if self.halt {
        let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
        println!("Press any key to resume execution...");
        stdin.read_line(&mut String::new()).unwrap();
        println!("... execution resumed.");
        let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
      }
    }
  }
}

impl<T: instructions::Set> fmt::Display for Cpu<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "  A={}   B={}   X={}   Y={}\n PC={}  SP={} HL={}\n  F={} ALU={}\n IR={}\nMEM={}",
      self.a, self.b, self.x, self.y, self.pc, self.sp,
      self.address, self.flags, self.alu, self.ir, self.memory)
  }
}

impl<T: instructions::Set> fmt::Debug for Cpu<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, " IR={:?}\n PC={:?}\n SP={:?}\n  A={:?}\n  B={:?}\n  X={:?}\n  Y={:?}\n HL={:?}\nMEM={:?}\n  F={:?}\nALU={:?}",
      self.ir, self.pc, self.sp, self.a, self.b, self.x, self.y,
      self.address, self.memory, self.flags, self.alu)
  }
}
