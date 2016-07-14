//! CHIT8 Emulator / disassembler executable for the library.

extern crate chip8;

use chip8::rom::Rom;
use std::fs::File;
use std::path::PathBuf;
use std::env;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

struct Options<>
{
	rom_path: PathBuf
}

fn usage()
{
	println!("CHIT8 emulator / disassembler {}", VERSION);
	println!("=====================================");
	println!("Usage: chit8 <path-to-rom>");
}

fn parse_cmdline_args() -> Option<Options>
{
	let mut opts = Options { rom_path: PathBuf::new() };

	for arg in env::args().skip(1) {
		match &*arg {
			_ => { opts.rom_path = PathBuf::from(arg) }
		} 
	}

	if !(opts.rom_path.is_file()) {
		usage();
		return None;
	}

	return Some(opts);
}

/// Loads the provided ROM and calls the library for disassembly.
pub fn main() {
	let opts = match parse_cmdline_args() { Some(opts) => opts, None => { return; } };
	let mut file = match File::open(&opts.rom_path) { Ok(x) => x, Err(err) =>  { println!("ROM Open error: {}", err.to_string()); return; }};

	let rom = match Rom::new(&mut file, opts.rom_path.file_name().unwrap_or_default().to_str().unwrap_or_default().to_owned()) { Ok(rom) => rom, Err(err) => { println!("ROM loading error: {}", err.to_string()); return; }};

	println!("ROM loaded: {}", rom);
    chip8::disasm(rom);
}
