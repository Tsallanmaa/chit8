//! Emulated RAM for the CHIP-8 computer. Contains loading and storing logic and 
//! initializes the work area with the provided ROM on initialization.

use rom::Rom;

/// Emulated RAM
pub struct Ram {
	/// RAM storage. CHIP-8 contains 4 kilobytes of RAM.
	mem: [u8; 0x1000]
}

impl Ram {
	/// Load a byte from RAM address $addr. Only the lowest 12 bits of the provided address byte
	/// are used. 
	pub fn lb(&mut self, addr: u16) -> u8 { self.mem[addr as usize & 0xFFF]}

	/// Store a byte to RAM at address $addr. Only the lowest 12 bits of the provided address byte 
	/// are used.
	pub fn sb(&mut self, addr: u16, value: u8) { self.mem[addr as usize & 0xFFF] = value; }

	/// Initialize a new RAM with the ROM provided copied into the work area at address 0x200 onwards.
	pub fn new(rom: &Rom) -> Ram 
	{ 
		let mut ram = Ram { mem: [0; 0x1000] };
		ram.mem[0x200..(0x200 + rom.length)].clone_from_slice(&rom.data[0..rom.length]);
		ram
	}
}