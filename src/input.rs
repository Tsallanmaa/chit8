//! Input module for the CHIP-8 emulation
//!
//! Provides the `Input` trait that provides the interface the emulator core
//! expects. 

use rand::{thread_rng, Rng};

/// `Input` -trait defines the input device the CHIP-8 emulation core expects.
/// It consists of reading key states.
pub trait Input {
	/// Returns an array of key states. Currently pressed keys have true as value,
	/// other keys have false.
	fn get_key_states(&self) -> [bool;16]; 
}

/// Emulated keyboard for the CHIP-8. Contains keys 0 to F in a numpad-like pattern.
#[allow(dead_code)]
pub struct Keyboard {
	keys: [bool;16]
}

impl Keyboard
{
	pub fn new() -> Keyboard
	{
		Keyboard { keys: [false;16] }
	}
}

impl Input for Keyboard
{
	fn get_key_states(&self) -> [bool;16]
	{
		let mut keys = [false; 16];
		for i in 0..16
		{
			keys[i] = thread_rng().gen();
		}
		keys
	}
}