//! ROM loaded into the CHIP-8 emulator / disassembler.

use std::io::{self, Read};
use std::fmt;

/// Struct describing the ROM file
pub struct Rom {
	/// File name of the ROM. Used for identification.
	pub filename: String,
	/// The actual ROM bytes. Maximum size that can fit into the CHIP-8 
	/// memory is 3232 bytes.
	pub data: [u8; 0xCA0],
	/// Length of the actual data loaded. 
	pub length: usize
}

impl Rom {
	/// Create a new ROM from the provided file and file name.
	/// File name is only used for identification and can be omitted.
	/// Only the first 3232 bytes of the ROM are loaded if a larger file is provided
	/// as this is the maximum number of bytes that can be copied into the CHIP-8 memory.
	pub fn new(readable: &mut Read, filename: String) -> Result<Rom, io::Error>
	{
		let mut buffer = [0u8; 0xCA0]; // Maximum size for ROMs is 3232 bytes
		let length = match readable.read(&mut buffer) { Ok(l) => l, Err(err) => return Err(err) };

		Ok(Rom { data: buffer, filename: filename, length: length })
	}
}

impl fmt::Display for Rom
{
	/// Implement fancy display formatting for the ROM
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "CHIP8 ROM ({}): {}KB",
        	self.filename,
            self.length
        )
    }
}