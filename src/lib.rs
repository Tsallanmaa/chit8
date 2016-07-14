//! Emulation and disassembly library for the CHIP-8 computer. 
//! Currently only contains a disassembler and primitive ROM loading.

pub mod rom;
pub mod disassembler;
pub mod ram;

use rom::Rom;
use ram::Ram;
use disassembler::Disassembler;

/// Disassemble the provided rom using the disassembler. Prints results to
/// the terminal.
pub fn disasm(rom: Rom)
{
	let mut dis = Disassembler { pc: 0x200, ram: &mut Ram::new(&rom) };
	dis.disasm(rom.length as u16);
}