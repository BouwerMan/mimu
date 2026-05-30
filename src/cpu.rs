use crate::assembler::Image;
use crate::instruction::Instruction;
use crate::instruction::decode;
use crate::register;
use crate::register::*;
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Cpu {
	registers: [u32; 32],
	pub pc: u32,
}

#[derive(Debug)]
pub struct Memory {
	data: HashMap<u32, u8>,
}

#[derive(Debug)]
pub struct Emulator {
	cpu: Cpu,
	memory: Memory,
}

#[derive(Error, Debug, PartialEq)]
pub enum ExecError {
	#[error("Unknown instruction")]
	UnknownInstruction,
}

#[derive(PartialEq)]
pub enum RunState {
	Running,
	Halted, // hit an exit syscall
}

impl Emulator {
	pub fn new() -> Self {
		Emulator {
			cpu: Cpu::new(),
			memory: Memory::new(),
		}
	}

	pub fn load(&mut self, img: &Image) {
		for (i, word) in img.text.iter().enumerate() {
			self.memory.write_word(i as u32 * 4, *word);
		}
	}

	pub fn step(&mut self) -> Result<RunState, ExecError> {
		let inst_raw = self.memory.read_word(self.cpu.pc); // Fetch instruction at current PC
		let inst = decode(inst_raw);
		self.cpu.pc = self.cpu.pc.wrapping_add(4);
		self.execute(inst)
	}

	pub fn run(&mut self) -> Result<(), ExecError> {
		while self.step()? == RunState::Running {}
		println!("{}", self.cpu); // Print CPU state after execution
		Ok(())
	}

	pub fn register(&mut self, n: usize) -> u32 {
		self.cpu.read_register(n)
	}

	fn execute(&mut self, inst: Instruction) -> Result<RunState, ExecError> {
		let mut next_state = RunState::Running;
		match inst {
			Instruction::LoadImmediate { rd, imm } => self.cpu.write_register(rd, imm as u32),
			Instruction::Add { rd, rs, rt } => {
				let rs_val = self.cpu.read_register(rs);
				let rt_val = self.cpu.read_register(rt);
				let result = rs_val.wrapping_add(rt_val); // Use wrapping_add to handle overflow
				self.cpu.write_register(rd, result);
			}
			Instruction::Addi { rt, rs, imm } => {
				let rs_val = self.cpu.read_register(rs);
				let result = rs_val.wrapping_add(imm as u32); // Use wrapping_add to handle overflow
				self.cpu.write_register(rt, result);
			}
			Instruction::Syscall => {
				let v0 = self.cpu.read_register(V0);
				match v0 {
					10 => {
						// exit syscall
						println!("Exit syscall called. Halting emulator.");
						next_state = RunState::Halted;
					}
					_ => unimplemented!("Syscall with $v0 = {} not implemented yet", v0),
				}
			}
			_ => unimplemented!("Instruction not implemented yet"),
		}
		Ok(next_state)
	}
}

impl Memory {
	fn new() -> Self {
		Memory {
			data: HashMap::new(),
		}
	}

	fn read_byte(&self, addr: u32) -> u8 {
		self.data.get(&addr).copied().unwrap_or(0)
	}

	pub fn read_word(&self, addr: u32) -> u32 {
		let mut word: u32 = 0;
		for byte in 0..4 {
			word |= (self.read_byte(addr + byte) as u32) << (byte * 8);
		}
		word
	}

	fn write_byte(&mut self, addr: u32, data: u8) {
		self.data.insert(addr, data);
	}

	pub fn write_word(&mut self, addr: u32, word: u32) {
		for byte in 0..4 {
			self.write_byte(addr + byte, (word >> (byte * 8)) as u8);
		}
	}
}

impl fmt::Display for Cpu {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		writeln!(f, "CPU State:")?;
		for (i, val) in self.registers.iter().enumerate() {
			write!(f, "${:<4}: {:>11}", register::NAMES[i], val)?;
			if i % 4 == 3 {
				writeln!(f)?; // end the row
			} else {
				write!(f, "    ")?; // gap to the next column
			}
		}
		Ok(())
	}
}

impl Cpu {
	pub fn new() -> Self {
		Cpu {
			registers: [0; 32],
			pc: 0,
		}
	}

	fn read_register(&self, reg: usize) -> u32 {
		self.registers[reg]
	}

	fn write_register(&mut self, reg: usize, value: u32) {
		if reg != 0 {
			self.registers[reg] = value;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::register::{T0, T1, T2};

	#[test]
	fn test_load_immediate() {
		let mut emu = Emulator::new();
		emu.execute(Instruction::LoadImmediate { rd: T0, imm: 42 }); // li $t0, 42
		assert_eq!(emu.register(T0), 42);
	}

	#[test]
	fn test_add() {
		let mut emu = Emulator::new();
		emu.execute(Instruction::LoadImmediate { rd: T0, imm: 10 }); // li $t0, 10
		emu.execute(Instruction::LoadImmediate { rd: T1, imm: 20 }); // li $t1, 20
		emu.execute(Instruction::Add {
			rd: T2,
			rs: T0,
			rt: T1,
		}); // add $t2, $t0, $t1
		assert_eq!(emu.register(T2), 30);
	}
}
