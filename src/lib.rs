//! Emulation and disassembly library for the CHIP-8 computer. 
//! Currently only contains a disassembler and primitive ROM loading.

extern crate rand;

#[macro_use]
pub mod disassembler;
pub mod rom;
pub mod ram;
pub mod cpu;
pub mod input;

use rom::Rom;
use ram::Ram;
use cpu::Cpu;
use input::Keyboard;
use disassembler::Disassembler;

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
	let mut ram = &mut Ram::new_from_rom(&rom);
	let keyboard = & Keyboard::new(); 
	let mut cpu = Cpu::new(ram, keyboard);
	loop {
		cpu.step();
	}
}