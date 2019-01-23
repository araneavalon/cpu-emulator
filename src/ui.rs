
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use crate::error::Error;
use crate::cpu::Cpu;
use crate::instructions;


const WIDTH: u32 = 240;
const HEIGHT: u32 = 128;
const SCALE: u32 = 4;


pub fn run<T: instructions::Set>(mut cpu: Cpu<T>) -> Result<(), Error> {
	let bg = Color::RGB(255, 255, 255);
	let fg = Color::RGB(  0,   0,   0);

	let sdl = sdl2::init().unwrap(); // TODO ERROR
	let video = sdl.video().unwrap(); // TODO ERROR
	let mut event_pump = sdl.event_pump().unwrap(); // TODO ERROR

	let window = video
		.window("cpu-emulator", WIDTH * SCALE, HEIGHT * SCALE)
		.position_centered()
		.build()
		.unwrap(); // TODO ERROR

	let mut canvas = window.into_canvas().build().unwrap(); // TODO ERROR

	canvas.set_scale(SCALE as f32, SCALE as f32).unwrap(); // TODO ERROR

	let ticks_per_frame = std::cmp::max(cpu.hz() / 60, 1);
	'running: loop {
		cpu.run(ticks_per_frame)?;

		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. } |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
				_ => (),
			}
		}

		canvas.set_draw_color(bg);
		canvas.clear();

		cpu.memory.io.screen.draw(&mut canvas, bg, fg)?;

		canvas.present();
	}

	Ok(())
}
