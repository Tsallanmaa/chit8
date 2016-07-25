//! Emulation and disassembly library for the CHIP-8 computer.

extern crate rand;
extern crate sdl2;

#[macro_use]
pub mod disassembler;
pub mod rom;
pub mod ram;
pub mod cpu;
pub mod input;
pub mod display;

use rom::Rom;
use ram::Ram;
use cpu::Cpu;
use display::SdlDisplay;
use input::Keyboard;
use disassembler::Disassembler;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

/// Disassemble the provided rom using the disassembler. Prints results to
/// the terminal.
pub fn disasm(rom: Rom)
{
	let mut dis = Disassembler { pc: 0x200, ram: &mut Ram::new_from_rom(&rom) };
	dis.disasm(rom.length as u16);
}

/// Start emulation on the provided rom.
pub fn emulate(rom: Rom)
{
	let sdl_context = sdl2::init().unwrap();
	let mut event_pump = sdl_context.event_pump().unwrap();

	let mut ram = &mut Ram::new_from_rom(&rom);
	let display = & SdlDisplay::new(sdl_context);
	let keyboard = & Keyboard::new(); 
	let mut cpu = Cpu::new(ram, keyboard, display);
	
	// Main emulator loop (Handle SDL, then do CPU loop)
	'mainloop: loop {
		for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'mainloop
                },
                _ => {}
            }
		}

		cpu.step();
	}
}