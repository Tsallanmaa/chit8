//! CPU module for CHIT8 emulator and disassembler.

use ram::*;
use input::Input;

use std::fmt;
use rand::{ThreadRng, thread_rng, Rng};

/// Emulated CPU of the CHIP-8
pub struct Cpu<'a, I: 'a + Input> {
	/// Main RAM (4 kilobytes)
	ram: &'a mut Memory,
	
	/// Program counter (PC)
	pc: u16,

	/// Registers V0 to VF
	v: [u8;16],

	/// Address register I
	i: u16,

	/// Stack
	/// CHIP-8 stack only contains return addresses for CALL opcodes. This
	/// implementation allows for 16 levels of nested CALL opcodes. 
	stack: [u16;16],

	/// Delay Timer (DT). Counts down at 60 Hz when value > 0
	dt: u8,

	/// Sound Timer (ST). Counts down at 60 Hz when value > 0
	st: u8,

	/// Random number generator
	rng: ThreadRng,

	/// Input device
	input: &'a I
}

impl<'a, I: Input> Cpu<'a, I>
{
	fn next_opcode(&mut self) -> u16
	{
		let hi = (self.ram.lb(self.pc) as u16) << 8;
		let low = self.ram.lb(self.pc+1) as u16;
		self.pc = self.pc + 2;
		low | hi
	}

	fn update_timers(&mut self)
	{
		if self.dt > 0
		{
			self.dt = self.dt - 1;
		}
		if self.st > 0
		{
			self.st = self.st - 1;
		}
	}

	/// Clear the display.
	fn cls(&mut self) 
	{
		// Display unimplemented
		return;
	}

	/// Return from a subroutine.
	/// The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
	fn ret(&mut self) 
	{
		if self.stack[0] == 0 { panic!("Return without anything on the stack!"); }
		
		let mut i = 0;
		while i < self.stack.len()
		{
			if self.stack[i] == 0
			{
				self.pc = self.stack[i-1];
				self.stack[i-1] = 0;
				return;
			}
			i = i + 1;
		}

		self.pc = self.stack[15];
		self.stack[15] = 0;
	}

	/// Jump to a machine code routine at addr.
	/// Commonly ignored.
	#[allow(unused_variables)]
	fn sys(&mut self, addr: u16)
	{
		// ignored
		return;
	}

	/// Jump to location addr.
	fn jp(&mut self, addr: u16)
	{
		self.pc = addr;
	}

	/// Call subroutine at addr.
	/// The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to addr.
	fn call(&mut self, addr: u16)
	{
		let mut i = 0;
		let mut found = false;
		while i < self.stack.len()
		{
			if self.stack[i] == 0 {
				self.stack[i] = self.pc; // Store PC address to stack
				found = true;
				break;
			}
			i = i + 1;
		}
		if !found {
			panic!("Call stack exceeded!");
		}

		self.pc = addr; // Jump to address
	}

	/// Skip next instruction if Vreg == val.
	fn se(&mut self, reg: u8, val: u8) 
	{
		if self.v[reg as usize] == val
		{
			self.pc = self.pc + 2; 
		}
	}

	/// Skip next instruction if Vreg != val.
	fn sne(&mut self, reg: u8, val: u8) 
	{
		if self.v[reg as usize] != val
		{
			self.pc = self.pc + 2;
		}
	}

	/// Skip next instruction if Vreg1 == Vreg2.
	fn se_reg(&mut self, reg1: u8, reg2: u8) 
	{
		if self.v[reg1 as usize] == self.v[reg2 as usize]
		{
			self.pc = self.pc + 2;
		}
	}

	/// Set Vreg = val.
	fn ldx(&mut self, reg: u8, val: u8)
	{
		self.v[reg as usize] = val;
	}

	/// Set Vreg = Vreg + byte.
	fn add_byte(&mut self, reg: u8, byte: u8)
	{
		self.v[reg as usize] = self.v[reg as usize].wrapping_add(byte); // CHIP-8 expects overflows
	}

	/// Set Vreg1 = Vreg2.
	fn ld(&mut self, reg1: u8, reg2: u8)
	{
		self.v[reg1 as usize] = self.v[reg2 as usize];
	}

	/// Set Vreg1 = Vreg1 || Vreg2.
	fn or(&mut self, reg1: u8, reg2: u8)
	{
		self.v[reg1 as usize] = self.v[reg1 as usize] | self.v[reg2 as usize];
	}

	/// Set Vreg1 = Vreg1 && Vreg2.
	fn and(&mut self, reg1: u8, reg2: u8) 
	{
		self.v[reg1 as usize] = self.v[reg1 as usize] & self.v[reg2 as usize];
	}

	/// Set Vreg1 = Vreg1 ^ Vreg2.
	fn xor(&mut self, reg1: u8, reg2: u8) 
	{
		self.v[reg1 as usize] = self.v[reg1 as usize] ^ self.v[reg2 as usize];
	}

	/// Set Vreg1 = Vreg1 + Vreg2, set VF = carry.
	/// The values of Vreg1 and Vreg2 are added together. If the result is greater than 8 bits, VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vreg1.
	fn add_reg(&mut self, reg1: u8, reg2: u8)
	{
		let v1 = self.v[reg1 as usize];
		let v2 = self.v[reg2 as usize];

		self.v[0xF] = if (v1 as u16) + (v2 as u16) > 0xFF { 1 } else { 0 }; // Carry flag to VF
		self.v[reg1 as usize] = v1.wrapping_add(v2);
	}

	/// Set Vreg1 = Vreg1 - Vreg2, set VF = NOT borrow.
	/// If Vreg1 > Vreg2, then VF is set to 1, otherwise 0. Then Vreg2 is subtracted from Vreg1, and the results stored in Vreg1.
	fn sub(&mut self, reg1: u8, reg2: u8) 
	{
		let v1 = self.v[reg1 as usize];
		let v2 = self.v[reg2 as usize];

		self.v[0xF] = if v1 > v2 { 1 } else { 0 }; // !borrow flag to VF
		self.v[reg1 as usize] = v1.wrapping_sub(v2);		
	}

	/// Set Vreg = Vreg SHR 1.
	/// If the least-significant bit of Vreg is 1, then VF is set to 1, otherwise 0. Then Vreg is divided by 2.
	fn shr(&mut self, reg: u8)
	{
		let val = self.v[reg as usize];

		self.v[0xF] = if 0b1 & val == 1 { 1 } else { 0 };
		self.v[reg as usize] = val >> 1;
	}

	/// Set Vreg1 = Vreg2 - Vreg1, set VF = NOT borrow.
	/// If Vreg2 > Vreg1, then VF is set to 1, otherwise 0. Then Vreg1 is subtracted from Vreg2, and the results stored in Vreg1.
	fn subn(&mut self, reg1: u8, reg2: u8) 
	{
		let v1 = self.v[reg1 as usize];
		let v2 = self.v[reg2 as usize];

		self.v[0xF] = if v2 > v1 { 1 } else { 0 }; // !borrow flag to VF
		self.v[reg1 as usize] = v2.wrapping_sub(v1);	
	}

	/// Set Vreg = Vreg SHL 1.
	/// If the most-significant bit of Vreg is 1, then VF is set to 1, otherwise to 0. Then Vreg is multiplied by 2.
	fn shl(&mut self, reg: u8)
	{
		let val = self.v[reg as usize];

		self.v[0xF] = if (0b10000000 & val) >> 7 == 1 { 1 } else { 0 };
		self.v[reg as usize] = val << 1;
	}

	/// Skip next instruction if Vreg1 != Vreg2.
	fn sne_reg(&mut self, reg1: u8, reg2: u8)
	{
		if self.v[reg1 as usize] != self.v[reg2 as usize]
		{
			self.pc = self.pc + 2;
		}
	}

	/// Set I = val.
	fn ldi(&mut self, val: u16)
	{
		self.i = val;
	}

	/// Jump to location addr + V0.
	fn jp_v0(&mut self, addr: u16)
	{
		self.pc = addr + (self.v[0] as u16);
	}

	/// Set Vreg = random byte && kk.
	fn rnd(&mut self, reg: u8, byte: u8)
	{
		self.v[reg as usize] = self.rng.gen::<u8>() & byte;
	}

	/// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
	///
	/// The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen. 
	/// If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen. 
	#[allow(unused_variables)]
	fn drw(&mut self, xreg: u8, yreg: u8, bytes: u8)
	{
		// unimplemented
		return;
	}

	/// Skip next instruction if key with the value of Vreg is pressed.
	fn skp(&mut self, reg: u8)
	{
		let state = self.input.get_key_states();
		let key = self.v[reg as usize];

		if state[key as usize] { self.pc = self.pc + 2; }
	}

	/// Skip next instruction if key with the value of Vreg is not pressed.
	fn sknp(&mut self, reg: u8)
	{
		let state = self.input.get_key_states();
		let key = self.v[reg as usize];

		if !state[key as usize] { self.pc = self.pc + 2; }
	}

	/// Set Vreg = delay timer value.
	fn ld_dt_into_vx(&mut self, reg: u8)
	{
		self.v[reg as usize] = self.dt;
	}

	/// Wait for a key press, store the value of the key in Vreg.
	fn ld_k_into_vx(&mut self, reg: u8)
	{
		loop {
			let state = self.input.get_key_states();
			for (index, value) in state.iter().enumerate()
			{
				if *value
				{
					self.v[reg as usize] = index as u8;
					return;
				}
			}
		}
	}

	/// Set delay timer = Vreg.
	fn ld_vx_into_dt(&mut self, reg: u8)
	{
		self.dt = self.v[reg as usize];
	}

	/// Set sound timer = Vreg.
	fn ld_vx_into_st(&mut self, reg: u8)
	{
		self.st = self.v[reg as usize];
	}

	/// Set I = I + Vreg.
	fn add_vx(&mut self, reg: u8)
	{
		self.i = self.i + self.v[reg as usize] as u16;
	}

	/// Set I = location of sprite for digit Vreg.
	/// The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vreg.
	fn ld_vx_digit_into_f(&mut self, reg: u8)
	{
		self.i = (self.v[reg as usize]*5) as u16; // 5 bytes per digit (starting from 0)
	}

	/// Store BCD representation of Vreg in memory locations I, I+1, and I+2.
	/// The interpreter takes the decimal value of Vreg, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
	fn ld_vx_into_bcd(&mut self, reg: u8)
	{
		let word = self.v[reg as usize].to_string();
		let mut chars = word.chars();
		let start_index = 3 - word.len(); // Starting index for actual digits
		let mut addr = self.i; // Copy, don't modify I

		for i in 0..3 {
			if i < start_index { 
				self.ram.sb(addr, 0x0); 
			} else {
				self.ram.sb(addr, chars.next().unwrap().to_digit(10).unwrap() as u8);
			}
			addr = addr + 1;
		}
	}

	/// Store registers V0 through Vreg in memory starting at location I.
	/// The interpreter copies the values of registers V0 through Vreg into memory, starting at the address in I.
	fn ld_v0_to_vx_into_i(&mut self, reg: u8)
	{
		let mut addr = self.i;

		for i in 0..reg+1
		{
			self.ram.sb(addr, self.v[i as usize]);
			addr = addr + 1;
		}
	}

	/// Read registers V0 through Vreg from memory starting at location I.
	/// The interpreter reads values from memory starting at location I into registers V0 through Vreg.
	fn ld_i_into_v0_to_vx(&mut self, reg: u8)
	{
		let mut addr = self.i;

		for i in 0..reg+1
		{
			self.v[i as usize] = self.ram.lb(addr);
			addr = addr + 1;
		}
	}

	/// Handler function for unknown opcodes.
	fn unknown_opcode(&mut self, op: u16)
	{
		println!("{}", self);
		panic!("Unknown opcode: 0x{:0>4X}", op)
	}

	pub fn step(&mut self)
	{
		let op = self.next_opcode();
		decode_opcode!(op, self);
		self.update_timers();
	}

	pub fn new<'b>(ram: &'b mut Memory, input: &'b I) -> Cpu<'b, I>
	{
		let rng = thread_rng();
		Cpu { ram: ram, pc: 0x200, v: [0;16], i:0, stack: [0;16], dt: 0, st: 0, rng: rng, input: input}
	}
}

impl<'a, I: Input> fmt::Display for Cpu<'a, I>
{
	/// Implement fancy display formatting for the CPU and it's state
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "CHIP8 CPU @ 0x{:0>4X}\r\n", self.pc));
        try!(write!(f, "V0: {:X}, V1: {:X}, V2: {:X}, V3: {:X}\r\n", self.v[0], self.v[1], self.v[2], self.v[3]));
        try!(write!(f, "V4: {:X}, V5: {:X}, V6: {:X}, V7: {:X}\r\n", self.v[4], self.v[5], self.v[6], self.v[7]));
        try!(write!(f, "V8: {:X}, V9: {:X}, VA: {:X}, VB: {:X}\r\n", self.v[8], self.v[9], self.v[0xA], self.v[0xB]));
        try!(write!(f, "VC: {:X}, VD: {:X}, VE: {:X}, VF: {:X}\r\n", self.v[0xC], self.v[0xD], self.v[0xE], self.v[0xF]));

        try!(write!(f, "\r\nSTACK:\r\n"));
        for (i, item) in self.stack.iter().enumerate()
        {
        	if *item == 0 { break; }
        	try!(write!(f, ">> {}: 0x{:0>4X}\r\n", i, item));
        }

        try!(write!(f, "\r\nI: {:X}", self.i));
        try!(write!(f, "\r\nST: {:X}", self.st));
        write!(f, "\r\nDT: {:X}", self.dt)
    }
}

// ---------
// - TESTS -
//----------

#[cfg(test)]
struct MockInput<'a> {
	keys: &'a mut [bool; 16]
}

#[cfg(test)]
impl<'a> MockInput<'a> {
	fn new(keys: &'a mut [bool; 16]) -> MockInput<'a> { MockInput { keys: keys } }
}

#[cfg(test)]
impl<'a> Input for MockInput<'a>
{
	fn get_key_states(&self) -> [bool;16] { self.keys.clone() }
}

#[test]
fn test_ret()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);

	cpu.pc = 0x200;
	cpu.stack[0] = 0xAFC;
	cpu.stack[1] = 0xBBB;
	
	cpu.ret();
	assert!(cpu.pc == 0xBBB); // Jumped to latest value on the stack
	for item in cpu.stack.iter().skip(1)
	{
		assert!(*item == 0x0)
	}

	cpu.ret();
	assert!(cpu.pc == 0xAFC); // Jumped to latest value on the stack
	for item in cpu.stack.iter()
	{
		assert!(*item == 0x0)
	}
}

#[test]
#[should_panic]
fn test_ret_panics_with_empty_stack()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);

	cpu.ret();
}

#[test]
fn test_jp()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.pc = 0x0;
	cpu.jp(0xABC);
	assert!(cpu.pc == 0xABC);

	cpu.jp(0xFAF);
	assert!(cpu.pc == 0xFAF);
}

#[test]
fn test_call()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);

	cpu.pc = 0x200;
	
	cpu.call(0xFFF);
	assert!(cpu.pc == 0xFFF); // PC after call is at PC
	assert!(cpu.stack[0] == 0x200); // PC before we called is on top of stack
	for item in cpu.stack.iter().skip(1)
	{
		assert!(*item == 0x0)
	}

	cpu.call(0xAAA);
	assert!(cpu.pc == 0xAAA); // New call, new PC
	assert!(cpu.stack[0] == 0x200); // nested call, oldest return address still at the top
	assert!(cpu.stack[1] == 0xFFF); // next return address at the next position
	for item in cpu.stack.iter().skip(2)
	{
		assert!(*item == 0x0)
	}
}

#[test]
#[should_panic]
fn test_call_overflows()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);

	for _ in  0..17 {
		cpu.call(0xFFF);
	}
}

#[test]
fn test_se()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0] = 0xAF;
	cpu.pc = 0x0;
	cpu.se(0x0, 0xAF);
	assert!(cpu.pc == 0x02); // Skipped one instruction

	cpu.se(0xF, 0xFF);
	assert!(cpu.pc == 0x02); // Register does not match, no skip
}

#[test]
fn test_sne()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0] = 0xAF;
	cpu.pc = 0x0;
	cpu.sne(0x0, 0xAF);
	assert!(cpu.pc == 0x00); // Skipped does match, no skip

	cpu.sne(0xF, 0xFF);
	assert!(cpu.pc == 0x02); // Register does match, skipped on opcode
}

#[test]
fn test_se_reg()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0] = 0xAF;
	cpu.v[0xA] = 0xFF;
	cpu.v[0x4] = 0xAF;
	cpu.pc = 0x0;

	cpu.se_reg(0x0, 0x4);
	assert!(cpu.pc == 0x02); // Skipped one instruction

	cpu.se_reg(0x4, 0x0);
	assert!(cpu.pc == 0x04); // Skipped one instruction

	cpu.se_reg(0x0, 0xA);
	assert!(cpu.pc == 0x04); // Registers do not match, no skip
}

#[test]
fn test_add_byte()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.add_byte(0xA, 0xFF);
	assert!(cpu.v[0xA] == 0xFF);

	cpu.add_byte(0xA, 0x09); // ADD should wrap properly
	assert!(cpu.v[0xA] == 0x08);

	cpu.add_byte(0xC, 0x04);
	assert!(cpu.v[0xC] == 0x04);
	assert!(cpu.v[0xA] == 0x08);
}

#[test]
fn test_ld()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0xF] = 0x34;
	cpu.ld(0xA, 0xF);
	assert!(cpu.v[0xA] == 0x34);
}

#[test]
fn test_ldx()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.ldx(0xA, 0xFF);
	assert!(cpu.v[0xA] == 0xFF);

	cpu.ldx(0x3, 0x21);
	assert!(cpu.v[0x3] == 0x21);
	assert!(cpu.v[0xA] == 0xFF);

	cpu.ldx(0xA, 0x02);
	assert!(cpu.v[0x3] == 0x21);
	assert!(cpu.v[0xA] == 0x02);
}

#[test]
fn test_or()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0xA] = 0xC;
	cpu.v[0xB] = 0x3;
	cpu.or(0xA, 0xB);
	assert!(cpu.v[0xA] == 0xC | 0x3);
	assert!(cpu.v[0xB] == 0x3);
}

#[test]
fn test_and()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0xA] = 0xC;
	cpu.v[0xB] = 0x3;
	cpu.and(0xA, 0xB);
	assert!(cpu.v[0xA] == 0xC & 0x3);
	assert!(cpu.v[0xB] == 0x3);
}

#[test]
fn test_xor()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0xA] = 0xC;
	cpu.v[0xB] = 0x3;
	cpu.xor(0xA, 0xB);
	assert!(cpu.v[0xA] == 0xC ^ 0x3);
	assert!(cpu.v[0xB] == 0x3);
}

#[test]
fn test_add_reg()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0xA] = 0xC;
	cpu.v[0xB] = 0x3;
	cpu.v[0xF] = 0xFF;
	cpu.add_reg(0xA, 0xB);
	assert!(cpu.v[0xA] == 0xC + 0x3);
	assert!(cpu.v[0xB] == 0x3);
	assert!(cpu.v[0xF] == 0x0); // VF = 0 since no overflow
}

#[test]
fn test_add_reg_overflows()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0xA] = 0xFA;
	cpu.v[0xB] = 0xAF;
	cpu.v[0xF] = 0xFF;
	cpu.add_reg(0xA, 0xB);
	assert!(cpu.v[0xA] == (0xFA as u8).wrapping_add(0xAF));
	assert!(cpu.v[0xB] == 0xAF);
	assert!(cpu.v[0xF] == 0x1); // VF = 1 since overflow occured
}

#[test]
fn test_sub()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0xA] = 0xC;
	cpu.v[0xB] = 0x3;
	cpu.v[0xF] = 0xFF;
	cpu.sub(0xA, 0xB);
	assert!(cpu.v[0xA] == 0xC - 0x3);
	assert!(cpu.v[0xB] == 0x3);
	assert!(cpu.v[0xF] == 0x1); // VF = 1 since no borrow and flag is !borrow
}

#[test]
fn test_sub_borrow()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0xA] = 0xAF;
	cpu.v[0xB] = 0xFA;
	cpu.v[0xF] = 0xFF;
	cpu.sub(0xA, 0xB);
	assert!(cpu.v[0xA] == (0xAF as u8).wrapping_sub(0xFA));
	assert!(cpu.v[0xB] == 0xFA);
	assert!(cpu.v[0xF] == 0x0); // VF = 0 since borrow occured and flag is !borrow
}

#[test]
fn test_shr()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0xA] = 0xFF;
	cpu.v[0xB] = 0x00;
	cpu.v[0xC] = 0x62;
	cpu.v[0xF] = 0xFF;

	cpu.shr(0xA);
	assert!(cpu.v[0xA] == 0xFF >> 1);
	assert!(cpu.v[0xF] == 0x1); // VF = 1 since lsb is 1

	cpu.shr(0xB);
	assert!(cpu.v[0xB] == 0x00 >> 1);
	assert!(cpu.v[0xF] == 0x0); // VF = 0 since lsb is 0

	cpu.v[0xF] = 0xFF;
	cpu.shr(0xC);
	assert!(cpu.v[0xC] == 0x62 >> 1); // 01100010 >> 00110001
	assert!(cpu.v[0xF] == 0x0); // VF = 0 since lsb is 0
}

#[test]
fn test_subn()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0xA] = 0x3;
	cpu.v[0xB] = 0xC;
	cpu.v[0xF] = 0xFF;
	cpu.subn(0xA, 0xB);
	assert!(cpu.v[0xA] == 0xC - 0x3);
	assert!(cpu.v[0xB] == 0xC);
	assert!(cpu.v[0xF] == 0x1); // VF = 1 since no borrow and flag is !borrow
}

#[test]
fn test_subn_borrow()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0xA] = 0xFA;
	cpu.v[0xB] = 0xAF;
	cpu.v[0xF] = 0xFF;
	cpu.subn(0xA, 0xB);
	assert!(cpu.v[0xA] == (0xAF as u8).wrapping_sub(0xFA));
	assert!(cpu.v[0xB] == 0xAF);
	assert!(cpu.v[0xF] == 0x0); // VF = 0 since borrow occured and flag is !borrow
}

#[test]
fn test_shl()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0xA] = 0xFF;
	cpu.v[0xB] = 0x00;
	cpu.v[0xC] = 0x62;
	cpu.v[0xF] = 0xFF;

	cpu.shl(0xA);
	assert!(cpu.v[0xA] == 0xFF << 1);
	assert!(cpu.v[0xF] == 0x1); // VF = 1 since msb is 1

	cpu.shl(0xB);
	assert!(cpu.v[0xB] == 0x00 << 1);
	assert!(cpu.v[0xF] == 0x0); // VF = 0 since msb is 0

	cpu.v[0xF] = 0xFF;
	cpu.shl(0xC);
	assert!(cpu.v[0xC] == 0x62 << 1); // 01100010 << 11000100
	assert!(cpu.v[0xF] == 0x0); // VF = 0 since msb is 0
}

#[test]
fn test_sne_reg()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);

	cpu.pc = 0x0;
	cpu.v[0xA] = 0x3;
	cpu.v[0xB] = 0xC;
	cpu.v[0xC] = 0xC;
	
	cpu.sne_reg(0xB, 0xC);
	assert!(cpu.pc == 0x0); // No skip because [0xB] == [0xC]

	cpu.sne_reg(0xA, 0xC); 
	assert!(cpu.pc == 0x2); // This skips

	cpu.sne_reg(0xC, 0xA);
	assert!(cpu.pc == 0x4); // So does this
}

#[test]
fn test_ldi()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);

	cpu.ldi(0xFFF);
	assert!(cpu.i == 0xFFF);

	cpu.ldi(0xACE);
	assert!(cpu.i == 0xACE);
}

#[test]
fn test_jp_v0()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);

	cpu.v[0] = 0xAC;
	cpu.jp_v0(0x21);
	assert!(cpu.pc == 0x21 + 0xAC);
}

#[test]
fn test_rnd()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0xA] = 0xFF;
	cpu.v[0x3] = 0xFF;
	cpu.v[0xD] = 0xFF;

	cpu.rnd(0xA, 0x00);
	assert!(cpu.v[0xA] == 0x00); // Always zero as mask is set

	cpu.rnd(0x3, 0xF0);
	assert!(cpu.v[0x3] & 0x0F == 0x00);

	cpu.rnd(0xD, 0x88);
	assert!(cpu.v[0xD] & 0b01110111 == 0x00);
}

#[test]
fn test_skp()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	keys[3] = true;
	keys[0xA] = true;

	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.pc = 0x0;
	cpu.v[0x0] = 3;
	cpu.v[0xC] = 0xF;
	cpu.v[0xD] = 0xA;

	cpu.skp(0x0); // Key directed to by register V0 has been pressed
	assert!(cpu.pc == 0x2);

	cpu.skp(0xC); // Key directed to by register VC has bot been pressed
	assert!(cpu.pc == 0x2);

	cpu.skp(0xD); // Key directed to by register VD has been pressed
	assert!(cpu.pc == 0x4);
}

#[test]
fn test_sknp()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	keys[3] = true;
	keys[0xA] = true;

	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.pc = 0x0;
	cpu.v[0x0] = 3;
	cpu.v[0xC] = 0xF;
	cpu.v[0xD] = 0xA;

	cpu.sknp(0x0); // Key directed to by register V0 has been pressed
	assert!(cpu.pc == 0x0);

	cpu.sknp(0xC); // Key directed to by register VC has bot been pressed
	assert!(cpu.pc == 0x2);

	cpu.sknp(0xD); // Key directed to by register VD has been pressed
	assert!(cpu.pc == 0x2);
}

#[test]
fn test_dt_into_vx()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.dt = 0xFF;
	cpu.ld_dt_into_vx(0);

	assert!(cpu.v[0] == 0xFF);

	cpu.dt = 0x30;
	cpu.ld_dt_into_vx(0x5);

	assert!(cpu.v[5] == 0x30);
}

#[test]
fn test_ld_k_into_vx()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	keys[0xA] = true;
	keys[0xB] = true;
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0xC] = 0xF;
	cpu.ld_k_into_vx(0xC);
	assert!(cpu.v[0xC] == 0xA); // Register set to first pressed key 
}

#[test]
fn test_ld_vx_into_dt()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0] = 3;
	cpu.ld_vx_into_dt(0);

	assert!(cpu.dt == 0x03);

	cpu.v[0xF] = 0xAE;
	cpu.ld_vx_into_dt(0xF);

	assert!(cpu.dt == 0xAE);
}

#[test]
fn test_ld_vx_into_st()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.v[0] = 3;
	cpu.ld_vx_into_st(0);

	assert!(cpu.st == 0x03);

	cpu.v[0xF] = 0xAE;
	cpu.ld_vx_into_st(0xF);

	assert!(cpu.st == 0xAE);
}

#[test]
fn test_add_vx()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.i = 0x2;
	cpu.v[0] = 0x3;
	cpu.add_vx(0);

	assert!(cpu.i == 0x2 + 0x3);

	cpu.v[0xF] = 0xAE;
	cpu.add_vx(0xF);

	assert!(cpu.i == 0x2 + 0x3 + 0xAE);
}

#[test]
fn test_ld_vx_digit_into_f()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.i = 0x0;
	cpu.v[0] = 3;
	cpu.ld_vx_digit_into_f(0);

	assert!(cpu.i == 0xF); // 15 bytes for digits 0, 1, 2 and 3 starts at 0xF

	cpu.i = 0x0;
	cpu.v[0xF] = 0xE;
	cpu.ld_vx_digit_into_f(0xF);

	assert!(cpu.i == 0x46); // 70 bytes for previous digits and F starts at 0x46
}

#[test]
fn test_ld_vx_into_bcd()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	cpu.i = 0x0;
	cpu.v[0] = 123;

	cpu.ld_vx_into_bcd(0);

	// Should result in 1 at I, 2 at I+1 and 3 at I+2
	assert!(cpu.ram.lb(cpu.i) == 1);
	assert!(cpu.ram.lb(cpu.i+1) == 2);
	assert!(cpu.ram.lb(cpu.i+2) == 3);
}

#[test]
fn test_ld_vx_into_bc_with_smaller_numbers()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	// Put some 0xFF:s into the memory to see writes
	cpu.ram.sb(cpu.i, 0xFF);
	cpu.ram.sb(cpu.i+1, 0xFF);
	cpu.ram.sb(cpu.i+2, 0xFF);

	cpu.i = 0x0;
	cpu.v[0xA] = 1;

	cpu.ld_vx_into_bcd(0xA);

	// Should result in 0 at I, 0 at I+1 and 1 at I+2
	assert!(cpu.ram.lb(cpu.i) == 0);
	assert!(cpu.ram.lb(cpu.i+1) == 0);
	assert!(cpu.ram.lb(cpu.i+2) == 1);
}

#[test]
fn test_ld_v0_to_vx_into_i()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	for i in 0..0x10 { cpu.v[i as usize] = i+1; }

	cpu.i = 0x0;
	cpu.ld_v0_to_vx_into_i(0xF);

	// Should result in memory containing numbers in rising value
	for i in 0..0x10
	{
		assert!(cpu.ram.lb(i) == (i+1) as u8);
	}
}

#[test]
fn test_ld_v0_to_vx_into_i_terminates_properly()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	for i in 0..0x10 { cpu.v[i as usize] = i+1; }

	cpu.i = 0x0;
	cpu.ld_v0_to_vx_into_i(0xA);

	// Should result in memory containing numbers in rising value
	for i in 0..0x10
	{
		assert!(cpu.ram.lb(i) == (if i <= 0xA { i+1 } else { 0 }) as u8);
	}
}

#[test]
fn test_ld_i_into_v0_to_vx()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	for i in 0..0xFF { cpu.ram.sb(i, i as u8); }

	cpu.i = 0x0;
	cpu.ld_i_into_v0_to_vx(0xF);

	// Should result in registers containing numbers in rising value
	for i in 0..0x10
	{
		assert!(cpu.v[i as usize] == i);
	}
}


#[test]
fn test_ld_i_into_v0_to_vx_terminates_properly()
{
	let mut ram = &mut Ram::new();
	let keys = &mut [false;16];
	let kb = & MockInput::new(keys);
	let mut cpu = Cpu::new(ram, kb);
	
	for i in 0..0xFF { cpu.ram.sb(i, i as u8); }

	cpu.i = 0x0;
	cpu.ld_i_into_v0_to_vx(0xA);

	// Should result registers containing numbers in rising value up to reg VA
	for i in 0..0x10
	{
		assert!(cpu.v[i as usize] == if i <= 0xA { i } else { 0 } );
	}
}