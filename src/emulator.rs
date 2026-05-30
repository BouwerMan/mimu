mod cpu;
mod memory;

use crate::asm::Image;
use crate::instruction::Instruction;
use crate::instruction::decode;
use crate::register;
use cpu::Cpu;
use memory::Memory;
use thiserror::Error;

const TEXT_START: u32 = 0x0040_0000;
const DATA_START: u32 = 0x1000_0000;

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

#[derive(Debug)]
pub struct Emulator {
	cpu: Cpu,
	memory: Memory,
}

impl Default for Emulator {
	fn default() -> Self {
		Self::new()
	}
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
			self.memory.write_word((i as u32 * 4) + TEXT_START, *word);
		}

		for (i, word) in img.data.iter().enumerate() {
			self.memory.write_byte((i as u32 * 4) + DATA_START, *word);
		}

		self.cpu.pc = TEXT_START + img.entry;
	}

	pub fn step(&mut self) -> Result<RunState, ExecError> {
		let inst_raw = self.memory.read_word(self.cpu.pc); // Fetch instruction at current PC
		let inst = decode(inst_raw);
		println!(
			"Executing instruction at PC={:#010x}: {:#010x} -> {:?}",
			self.cpu.pc, inst_raw, inst
		); // Debug print
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
			Instruction::Add { rd, rs, rt } => {
				let rs_val = self.cpu.read_register(rs);
				let rt_val = self.cpu.read_register(rt);
				let result = rs_val.wrapping_add(rt_val); // Use wrapping_add to handle overflow
				self.cpu.write_register(rd, result);
			}
			Instruction::AddImmediate { rt, rs, imm } => {
				let rs_val = self.cpu.read_register(rs);
				let result = rs_val.wrapping_add(imm as u32); // Use wrapping_add to handle overflow
				self.cpu.write_register(rt, result);
			}
			Instruction::Syscall => {
				let v0 = self.cpu.read_register(register::V0);
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::register::{T0, T1, T2};

	#[test]
	fn test_add() {
		let mut emu = Emulator::new();
		emu.execute(Instruction::AddImmediate {
			rt: T0,
			rs: register::ZERO,
			imm: 10,
		});
		emu.execute(Instruction::AddImmediate {
			rt: T1,
			rs: register::ZERO,
			imm: 20,
		});
		emu.execute(Instruction::Add {
			rd: T2,
			rs: T0,
			rt: T1,
		}); // add $t2, $t0, $t1
		assert_eq!(emu.register(T2), 30);
	}
}
