//! Disassembly module for CHIT8 emulator and disassembler.
//!
//! Contains `decode_opcode!` macro to decode opcode and the parameters for use in the emulated CPU
//! and the disassembler. The disassembler contains definitions for all these opcodes that provide a 
//! string representation of the opcode and it's parameters. 

use ram::Ram;

/// Macro to decode opcode and call the corresponsing function on the emulated CPU or disassembler
/// with the correct parameters parsed from the opcode.
///
/// Source for the opcodes: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
macro_rules! decode_opcode {
	($op:expr, $this:ident) => {
		match $op {
			0x00E0 => { $this.cls() },
			0x00EE => { $this.ret() },
			op @ 0x0000 ... 0x0FFF => { $this.sys(op & 0xFFF) },
			op @ 0x1000 ... 0x1FFF => { $this.jp(op & 0x0FFF) },
			op @ 0x2000 ... 0x2FFF => { $this.call(op & 0x0FFF) },
			op @ 0x3000 ... 0x3FFF => { $this.se(((op & 0x0F00) >> 8) as u8, (op & 0x00FF) as u8) },
			op @ 0x4000 ... 0x4FFF => { $this.sne(((op & 0x0F00) >> 8) as u8, (op & 0x00FF) as u8) },
			op @ 0x5000 ... 0x5FFF => { $this.se_reg(((op & 0x0F00) >> 8) as u8, ((op & 0x00F0) >> 4) as u8) },
			op @ 0x6000 ... 0x6FFF => { $this.ldx(((op & 0x0F00) >> 8) as u8, (op & 0x00FF) as u8) },
			op @ 0x7000 ... 0x7FFF => { $this.add_byte(((op & 0x0F00) >> 8) as u8, (op & 0x00FF) as u8) },
			op @ 0x8000 ... 0x8FFF if (op & 0x000F) == 0x0 => { $this.ld(((op & 0x0F00) >> 8) as u8, ((op & 0x00F0) >> 4) as u8) },
			op @ 0x8000 ... 0x8FFF if (op & 0x000F) == 0x1 => { $this.or(((op & 0x0F00) >> 8) as u8, ((op & 0x00F0) >> 4) as u8) },
			op @ 0x8000 ... 0x8FFF if (op & 0x000F) == 0x2 => { $this.and(((op & 0x0F00) >> 8) as u8, ((op & 0x00F0) >> 4) as u8) },
			op @ 0x8000 ... 0x8FFF if (op & 0x000F) == 0x3 => { $this.xor(((op & 0x0F00) >> 8) as u8, ((op & 0x00F0) >> 4) as u8) },
			op @ 0x8000 ... 0x8FFF if (op & 0x000F) == 0x4 => { $this.add_reg(((op & 0x0F00) >> 8) as u8, ((op & 0x00F0) >> 4) as u8) },
			op @ 0x8000 ... 0x8FFF if (op & 0x000F) == 0x5 => { $this.sub(((op & 0x0F00) >> 8) as u8, ((op & 0x00F0) >> 4) as u8) },
			op @ 0x8000 ... 0x8FFF if (op & 0x000F) == 0x6 => { $this.shr(((op & 0x0F00) >> 8) as u8) },
			op @ 0x8000 ... 0x8FFF if (op & 0x000F) == 0x7 => { $this.subn(((op & 0x0F00) >> 8) as u8, ((op & 0x00F0) >> 4) as u8) },
			op @ 0x8000 ... 0x8FFF if (op & 0x000F) == 0xE => { $this.shl(((op & 0x0F00) >> 8) as u8) },
			op @ 0x9000 ... 0x9FFF if (op & 0x000F) == 0x0 => { $this.sne_reg(((op & 0x0F00) >> 8) as u8, ((op & 0x00F0) >> 4) as u8) },
			op @ 0xA000 ... 0xAFFF => { $this.ldi(op & 0x0FFF)},
			op @ 0xB000 ... 0xBFFF => { $this.jp_v0(op & 0x0FFF)},
			op @ 0xC000 ... 0xCFFF => { $this.rnd(((op & 0x0F00) >> 8) as u8, (op & 0x00FF) as u8) },
			op @ 0xD000 ... 0xDFFF => { $this.drw(((op & 0x0F00) >> 8) as u8, ((op & 0x00F0) >> 4) as u8, (op & 0x000F) as u8) },
			op @ 0xE000 ... 0xEFFF if (op & 0x00FF) == 0x9E => { $this.skp(((op & 0x0F00) >> 8) as u8) },
			op @ 0xE000 ... 0xEFFF if (op & 0x00FF) == 0xA1 => { $this.sknp(((op & 0x0F00) >> 8) as u8) },
			op @ 0xF000 ... 0xFFFF if (op & 0x00FF) == 0x07 => { $this.ld_dt_into_vx(((op & 0x0F00) >> 8) as u8) },
			op @ 0xF000 ... 0xFFFF if (op & 0x00FF) == 0x0A => { $this.ld_k_into_vx(((op & 0x0F00) >> 8) as u8) },
			op @ 0xF000 ... 0xFFFF if (op & 0x00FF) == 0x15 => { $this.ld_vx_into_dt(((op & 0x0F00) >> 8) as u8) },
			op @ 0xF000 ... 0xFFFF if (op & 0x00FF) == 0x18 => { $this.ld_vx_into_st(((op & 0x0F00) >> 8) as u8) },
			op @ 0xF000 ... 0xFFFF if (op & 0x00FF) == 0x1E => { $this.add_vx(((op & 0x0F00) >> 8) as u8) },
			op @ 0xF000 ... 0xFFFF if (op & 0x00FF) == 0x29 => { $this.ld_vx_digit_into_f(((op & 0x0F00) >> 8) as u8) },
			op @ 0xF000 ... 0xFFFF if (op & 0x00FF) == 0x33 => { $this.ld_vx_into_bcd(((op & 0x0F00) >> 8) as u8) },
			op @ 0xF000 ... 0xFFFF if (op & 0x00FF) == 0x55 => { $this.ld_v0_to_vx_into_i(((op & 0x0F00) >> 8) as u8) },
			op @ 0xF000 ... 0xFFFF if (op & 0x00FF) == 0x65 => { $this.ld_i_into_v0_to_vx(((op & 0x0F00) >> 8) as u8) },
			_ => $this.unknown_opcode($op)
		}
	}
}

/// Disassembler for the CHIP-8. Comments for the emulated opcodes are
/// sourced from http://devernay.free.fr/hacks/chip8/C8TECH10.HTM and modified.
pub struct Disassembler<'a>
{
	/// Current program counter. Initialized to 0x200.
	pub pc: u16,
	/// Emulated RAM of the CHIP-8
	pub ram: &'a mut Ram
}

impl<'a> Disassembler<'a> {
	/// Fetches the next opcode from memory and returns it and 
	/// the program counter for the opcode in a tuple.
	fn next_opcode(&mut self) -> (u16, u16)
	{
		let hi = (self.ram.lb(self.pc) as u16) << 8;
		let low = self.ram.lb(self.pc+1) as u16;
		let pc = self.pc;
		self.pc = self.pc + 2;
		(pc, low | hi)
	}

	/// Clear the display.
	fn cls(&mut self) -> String
	{
		"CLS".to_string()
	}

	/// Return from a subroutine.
	/// The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
	fn ret(&mut self) -> String
	{
		"RET".to_string()
	}

	/// Jump to a machine code routine at addr.
	/// Commonly ignored.
	fn sys(&mut self, addr: u16) -> String
	{
		format!("SYS 0x{:0>4X}", addr)
	}

	/// Jump to location addr.
	fn jp(&mut self, addr: u16) -> String
	{
		format!("JP {:#X}", addr)
	}

	/// Call subroutine at addr.
	/// The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to addr.
	fn call(&mut self, addr: u16) -> String
	{
		format!("CALL {:#X}", addr)
	}

	/// Skip next instruction if Vreg == val.
	fn se(&mut self, reg: u8, val: u8) -> String 
	{
		format!("SE V{:X}, {:X}", reg, val)
	}

	/// Skip next instruction if Vreg != val.
	fn sne(&mut self, reg: u8, val: u8) -> String 
	{
		format!("SNE V{:X}, {:X}", reg, val)
	}

	/// Skip next instruction if Vreg1 == Vreg2.
	fn se_reg(&mut self, reg1: u8, reg2: u8) -> String 
	{
		format!("SE V{:X}, V{:X}", reg1, reg2)
	}

	/// Set Vreg = val.
	fn ldx(&mut self, reg: u8, val: u8) -> String
	{
		format!("LD V{:X}, {:#X}", reg, val)
	}

	/// Set Vreg = Vreg + byte.
	fn add_byte(&mut self, reg: u8, byte: u8) -> String
	{
		format!("ADD V{:X}, {:X}", reg, byte)
	}

	/// Set Vreg1 = Vreg2.
	fn ld(&mut self, reg1: u8, reg2: u8) -> String
	{
		format!("LD V{:X}, V{:X}", reg1, reg2)
	}

	/// Set Vreg1 = Vreg1 || Vreg2.
	fn or(&mut self, reg1: u8, reg2: u8) -> String
	{
		format!("OR V{:X}, V{:X}", reg1, reg2)
	}

	/// Set Vreg1 = Vreg1 && Vreg2.
	fn and(&mut self, reg1: u8, reg2: u8) -> String
	{
		format!("AND V{:X}, V{:X}", reg1, reg2)
	}

	/// Set Vreg1 = Vreg1 ^ Vreg2.
	fn xor(&mut self, reg1: u8, reg2: u8) -> String
	{
		format!("XOR V{:X}, V{:X}", reg1, reg2)
	}

	/// Set Vreg1 = Vreg1 + Vreg2, set VF = carry.
	/// The values of Vreg1 and Vreg2 are added together. If the result is greater than 8 bits, VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vreg1.
	fn add_reg(&mut self, reg1: u8, reg2: u8) -> String
	{
		format!("ADD V{:X}, V{:X}", reg1, reg2)
	}

	/// Set Vreg1 = Vreg1 - Vreg2, set VF = NOT borrow.
	/// If Vreg1 > Vreg2, then VF is set to 1, otherwise 0. Then Vreg2 is subtracted from Vreg1, and the results stored in Vreg1.
	fn sub(&mut self, reg1: u8, reg2: u8) -> String
	{
		format!("SUB V{:X}, V{:X}", reg1, reg2)
	}

	/// Set Vreg = Vreg SHR 1.
	/// If the least-significant bit of Vreg is 1, then VF is set to 1, otherwise 0. Then Vreg is divided by 2.
	fn shr(&mut self, reg: u8) -> String
	{
		format!("SHR V{:X}", reg)
	}

	/// Set Vreg1 = Vreg2 - Vreg1, set VF = NOT borrow.
	/// If Vreg2 > Vreg1, then VF is set to 1, otherwise 0. Then Vreg1 is subtracted from Vreg2, and the results stored in Vreg1.
	fn subn(&mut self, reg1: u8, reg2: u8) -> String
	{
		format!("SUBN V{:X}, V{:X}", reg1, reg2)
	}

	/// Set Vreg = Vreg SHL 1.
	/// If the most-significant bit of Vreg is 1, then VF is set to 1, otherwise to 0. Then Vreg is multiplied by 2.
	fn shl(&mut self, reg: u8) -> String
	{
		format!("SHR V{:X}", reg)
	}

	/// Skip next instruction if Vreg1 != Vreg2.
	fn sne_reg(&mut self, reg1: u8, reg2: u8) -> String
	{
		format!("SNE V{:X}, V{:X}", reg1, reg2)
	}

	/// Set I = val.
	fn ldi(&mut self, val: u16) -> String
	{
		format!("LD I, {:#X}", val)
	}

	/// Jump to location addr + V0.
	fn jp_v0(&mut self, addr: u16) -> String
	{
		format!("JP V0, {:#X}", addr)
	}

	/// Set Vreg = random byte && kk.
	fn rnd(&mut self, reg: u8, byte: u8) -> String
	{
		format!("RND V{:X}, {:X}", reg, byte)
	}

	/// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
	///
	/// The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen. 
	/// If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen. 
	fn drw(&mut self, xreg: u8, yreg: u8, bytes: u8) -> String
	{
		format!("DRW (V{:X}, V{:X}) for {:X} bytes", xreg, yreg, bytes)
	}

	/// Skip next instruction if key with the value of Vreg is pressed.
	fn skp(&mut self, reg: u8) -> String
	{
		format!("SKP V{:X}", reg)
	}

	/// Skip next instruction if key with the value of Vreg is not pressed.
	fn sknp(&mut self, reg: u8) -> String
	{
		format!("SKNP V{:X}", reg)
	}

	/// Set Vreg = delay timer value.
	fn ld_dt_into_vx(&mut self, reg: u8) -> String
	{
		format!("LD V{:X}, DT", reg)
	}

	/// Wait for a key press, store the value of the key in Vreg.
	fn ld_k_into_vx(&mut self, reg: u8) -> String
	{
		format!("LD V{:X}, K", reg)
	}

	/// Set delay timer = Vreg.
	fn ld_vx_into_dt(&mut self, reg: u8) -> String
	{
		format!("LD DT, V{:X}", reg)
	}

	/// Set sound timer = Vreg.
	fn ld_vx_into_st(&mut self, reg: u8) -> String
	{
		format!("LD ST, V{:X}", reg)
	}

	/// Set I = I + Vreg.
	fn add_vx(&mut self, reg: u8) -> String
	{
		format!("ADD I, V{:X}", reg)
	}

	/// Set I = location of sprite for digit Vreg.
	/// The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vreg.
	fn ld_vx_digit_into_f(&mut self, reg: u8) -> String
	{
		format!("LD F, V{:X}", reg)
	}

	/// Store BCD representation of Vreg in memory locations I, I+1, and I+2.
	/// The interpreter takes the decimal value of Vreg, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
	fn ld_vx_into_bcd(&mut self, reg: u8) -> String
	{
		format!("LD B, V{:X}", reg)
	}

	/// Store registers V0 through Vreg in memory starting at location I.
	/// The interpreter copies the values of registers V0 through Vreg into memory, starting at the address in I.
	fn ld_v0_to_vx_into_i(&mut self, reg: u8) -> String
	{
		format!("LD [I], V{:X}", reg)
	}

	/// Read registers V0 through Vreg from memory starting at location I.
	/// The interpreter reads values from memory starting at location I into registers V0 through Vreg.
	fn ld_i_into_v0_to_vx(&mut self, reg: u8) -> String
	{
		format!("LD V{:X}, [I]", reg)
	}

	/// Handler function for unknown opcodes.
	fn unknown_opcode(&mut self, op: u16) -> String
	{
		format!("Unknown opcode: 0x{:0>4X}", op)
	}

	/// Run the disassembly and print the results.
	/// Runs until program counter reaches the end of the ROM.
	pub fn disasm(&mut self, rom_length: u16) {
		println!("");
		println!("===");

		loop {
			let op = self.next_opcode();
			println!("{:#X}: (0x{:0>4X}) {}", op.0, op.1, decode_opcode!(op.1, self));
			if self.pc >= (0x200 + rom_length) { break; }
		}
	}
}