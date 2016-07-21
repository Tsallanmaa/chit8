//! Emulated RAM for the CHIP-8 computer. Contains loading and storing logic and 
//! initializes the work area with the provided ROM on initialization.

use rom::Rom;

/// The data for the CHIP-8 font set. Each digit
/// is 4 pixels wide and 5 pixels high, resulting
/// in 5 bytes of data for each digit.
static FONT_DATA: &'static [u8] = & [
  0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
  0x20, 0x60, 0x20, 0x20, 0x70, // 1
  0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
  0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
  0x90, 0x90, 0xF0, 0x10, 0x10, // 4
  0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
  0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
  0xF0, 0x10, 0x20, 0x40, 0x40, // 7
  0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
  0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
  0xF0, 0x90, 0xF0, 0x90, 0x90, // A
  0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
  0xF0, 0x80, 0x80, 0x80, 0xF0, // C
  0xE0, 0x90, 0x90, 0x90, 0xE0, // D
  0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
  0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

/// Memory trait provides the interface for memory implementations. Currently there is only the 
/// `Ram` implementation.
pub trait Memory {
	/// Load a byte from RAM address $addr. Only the lowest 12 bits of the provided address byte
	/// are used. 
	fn lb(&mut self, addr: u16) -> u8;

	/// Store a byte to RAM at address $addr. Only the lowest 12 bits of the provided address byte 
	/// are used.
	fn sb(&mut self, addr: u16, value: u8);
}

/// Emulated RAM
pub struct Ram {
	/// RAM storage. CHIP-8 contains 4 kilobytes of RAM.
	mem: [u8; 0x1000]
}

impl Ram {
	/// Initialize a new RAM with the ROM provided copied into the work area at address 0x200 onwards.
	pub fn new_from_rom(rom: &Rom) -> Ram 
	{ 
		let mut ram = Ram { mem: [0; 0x1000] };
		ram.mem[0x000..0x050].clone_from_slice(&FONT_DATA[..]);
		ram.mem[0x200..(0x200 + rom.length)].clone_from_slice(&rom.data[0..rom.length]);
		ram
	}

	/// Initialize new empty RAM
	pub fn new() -> Ram 
	{ 
		Ram { mem: [0; 0x1000] }
	}
}

impl Memory for Ram {
	/// Load a byte from RAM address $addr. Only the lowest 12 bits of the provided address byte
	/// are used. 
	fn lb(&mut self, addr: u16) -> u8 { self.mem[addr as usize & 0xFFF]}

	/// Store a byte to RAM at address $addr. Only the lowest 12 bits of the provided address byte 
	/// are used.
	fn sb(&mut self, addr: u16, value: u8) { self.mem[addr as usize & 0xFFF] = value; }
}