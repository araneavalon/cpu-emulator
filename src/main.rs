
extern crate assembler;
extern crate sdl2;
extern crate backtrace;

mod error;
mod components;
mod memory;
mod io;
mod control;
mod cpu;

use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::{
  thread,
  time::Duration,
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
const HZ:  f64 = 48.0;

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
fn assemble_rom(filename: &str) -> Vec<u16> {
  assembler::from_file(filename).unwrap() // TODO ERROR
}

fn run(cpu: &mut Cpu) -> Result<()> {
  let half_cycle = Duration::from_millis((1000.0 / (HZ * 2.0)) as u64);

  let bg = Color::RGB(255, 255, 255);
  let fg = Color::RGB(  0,   0,   0);

  let sdl = sdl2::init().unwrap(); // TODO ERROR
  let video = sdl.video().unwrap(); // TODO ERROR
  let mut event_pump = sdl.event_pump().unwrap(); // TODO ERROR

  let pixel_w = (WIDTH + ((BORDER as u32) * 2)) * (SCALE as u32);
  let pixel_h = (HEIGHT + ((BORDER as u32) * 2)) * (SCALE as u32);
  let window = video
    .window("cpu-emulator", pixel_w, pixel_h)
    .position_centered()
    .build()
    .unwrap(); // TODO ERROR
  let mut canvas = window.into_canvas().build().unwrap(); // TODO ERROR
  canvas.set_scale(SCALE, SCALE).unwrap(); // TODO ERROR
  canvas.set_viewport(Rect::new(BORDER, BORDER, WIDTH, HEIGHT));

  let ticks_per_frame = std::cmp::max((HZ / FPS) as usize, 1);

  let mut halt = false;
  'running: loop {
    if !halt {
      for _ in 0..ticks_per_frame {
        thread::sleep(half_cycle);
        halt = cpu.half_cycle()?;
        thread::sleep(half_cycle);
        cpu.cycle()?;
        println!("{}", cpu);
        if halt { break }
      }
    } else {
      thread::sleep(half_cycle * (ticks_per_frame as u32) * 2);
    }

    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. } |
        Event::KeyDown { keycode: Some(Keycode::ScrollLock), .. } => break 'running,
        Event::KeyDown { keycode: Some(Keycode::Pause), .. } => halt = !halt,
        Event::KeyDown { keycode: Some(key), keymod, .. } => {
          if cpu.keyboard()?.pressed(key, keymod) {
            println!("Keyboard Interrupt: {}, {:?}, {:?}", KB_INT, key, keymod);
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
  let mut args = env::args();

  let assemble = match args.nth(1) {
    None => return Err(Error::InvalidROM),
    Some(flag) => {
      match flag.as_ref() {
        "-r" => false,
        "-a" => true,
        _ => return Err(Error::InvalidROM),
      }
    },
  };
  let filename = match args.next() {
    None => return Err(Error::InvalidROM),
    Some(filename) => filename,
  };
  let rom = if assemble {
    assemble_rom(&filename) // TODO ERROR
  } else {
    match load_rom(&filename) {
      Err(error) => return Err(Error::File(filename, error)),
      Ok(rom) => rom,
    }
  };

  let mut cpu = Cpu::new(rom)?;
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
