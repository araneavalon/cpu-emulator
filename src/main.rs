
extern crate assembler;
extern crate sdl2;
extern crate clap;

#[macro_use]
mod error;
mod components;
mod memory;
mod io;
mod control;
mod cpu;

use std::io::prelude::*;
use std::fs::File;
use clap::{
  App,
  Arg,
};

use sdl2::{
  pixels::Color,
  event::Event,
  keyboard::Keycode,
  rect::Rect,
};

use crate::error::{
  Result,
  Error,
};
use crate::cpu::Cpu;


const FPS: f64 = 12.0;
const DEFAULT_HZ: &'static str = "48.0";

const WIDTH:  u32 = 240;
const HEIGHT: u32 = 128;
const BORDER: i32 = 1;
const SCALE:  f32 = 4.0;

const KB_INT: u16 = 6;


fn load_rom(filename: &str) -> std::io::Result<Vec<u16>> {
  let mut file = Vec::new();
  File::open(filename)?.read_to_end(&mut file)?;

  let words = file.chunks(2)
    .map(|chunk| {
      match chunk {
        &[l, h] => u16::from_le_bytes([l, h]),
        _ => panic!("Invalid ROM file: Incomplete word."),
      }
    })
    .collect::<Vec<u16>>();

  Ok(words)
}

fn run(cpu: &mut Cpu) -> Result<()> {
  let bg = Color::RGB(255, 255, 255);
  let fg = Color::RGB(  0,   0,   0);

  let sdl = sdl_e!(sdl2::init())?;
  let video = sdl_e!(sdl.video())?;
  let mut event_pump = sdl_e!(sdl.event_pump())?;

  let pixel_w = (WIDTH + ((BORDER as u32) * 2)) * (SCALE as u32);
  let pixel_h = (HEIGHT + ((BORDER as u32) * 2)) * (SCALE as u32);
  let window = sdl_e!(video
    .window("cpu-emulator", pixel_w, pixel_h)
    .position_centered()
    .build())?;
  let mut canvas = sdl_e!(window.into_canvas().build())?;
  sdl_e!(canvas.set_scale(SCALE, SCALE))?;
  canvas.set_viewport(Rect::new(BORDER, BORDER, WIDTH, HEIGHT));

  let cycles_per_frame = std::cmp::max((cpu.hz() / FPS) as u32, 1);
  'running: loop {
    cpu.run(cycles_per_frame)?;

    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. } |
        Event::KeyDown { keycode: Some(Keycode::ScrollLock), .. } => break 'running,
        Event::KeyDown { keycode: Some(Keycode::Pause), .. } => cpu.pause(),
        Event::KeyDown { keycode: Some(key), keymod, .. } => {
          if cpu.keyboard()?.pressed(key, keymod) {
            cpu.interrupt(KB_INT)?;
          }
        },
        _ => (),
      }
    }

    canvas.set_draw_color(bg);
    canvas.clear();

    cpu.screen()?.draw(&mut canvas, bg, fg)?;

    canvas.present();
  }

  Ok(())
}

fn init() -> Result<()> {
  let args = App::new("cpu-emulator")
    .arg(Arg::with_name("asm")
      .long("asm")
      .short("a")
      .takes_value(true)
      .conflicts_with("rom")
      .required_unless("rom"))
    .arg(Arg::with_name("rom")
      .long("rom")
      .short("r")
      .takes_value(true)
      .required_unless("asm"))
    .arg(Arg::with_name("hz")
      .long("hz")
      .short("c")
      .takes_value(true))
    .get_matches();

  let hz = args.value_of("hz").unwrap_or(DEFAULT_HZ).parse::<f64>()?;
  let rom = if args.is_present("asm") {
    match args.value_of("asm") {
      None => return Err(Error::InvalidROM),
      Some(filename) => {
        match assembler::from_file(filename) {
          Err(error) => return Err(Error::Assembler(String::from(filename), error)),
          Ok(rom) => rom,
        }
      },
    }
  } else if args.is_present("rom") {
    match args.value_of("rom") {
      None => return Err(Error::InvalidROM),
      Some(filename) => {
        match load_rom(filename) {
          Err(error) => return Err(Error::File(String::from(filename), error)),
          Ok(rom) => rom,
        }
      },
    }
  } else {
    return Err(Error::InvalidROM)
  };

  let mut cpu = Cpu::new(hz, rom)?;
  let result = run(&mut cpu);
  if let Err(_) = result {
    println!("\n\nLast CPU State:\n{}", cpu);
  }
  result
}

fn main() {
  if let Err(error) = init() {
    eprintln!("Error:\n\t{}", error);
    std::process::exit(1)
  }
}
