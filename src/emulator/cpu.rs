use crate::register;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Cpu {
	registers: [u32; 32],
	pub pc: u32,
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

	pub fn read_register(&self, reg: usize) -> u32 {
		self.registers[reg]
	}

	pub fn write_register(&mut self, reg: usize, value: u32) {
		if reg != 0 {
			self.registers[reg] = value;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::register;

	#[test]
	fn new_zeroes_all_registers_and_pc() {
		let cpu = Cpu::new();
		assert_eq!(cpu.pc, 0);
		for i in 0..32 {
			assert_eq!(cpu.read_register(i), 0);
		}
	}

	#[test]
	fn write_then_read_register() {
		let mut cpu = Cpu::new();
		cpu.write_register(register::T0, 42);
		assert_eq!(cpu.read_register(register::T0), 42);
	}

	#[test]
	fn write_to_zero_register_is_ignored() {
		let mut cpu = Cpu::new();
		cpu.write_register(register::ZERO, 0xDEAD_BEEF);
		assert_eq!(cpu.read_register(register::ZERO), 0);
	}

	#[test]
	fn write_max_u32() {
		let mut cpu = Cpu::new();
		cpu.write_register(register::S0, u32::MAX);
		assert_eq!(cpu.read_register(register::S0), u32::MAX);
	}

	#[test]
	fn write_does_not_affect_other_registers() {
		let mut cpu = Cpu::new();
		cpu.write_register(register::T1, 99);
		for i in 1..32 {
			if i != register::T1 {
				assert_eq!(cpu.read_register(i), 0, "register {i} unexpectedly modified");
			}
		}
	}

	#[test]
	fn overwrite_register() {
		let mut cpu = Cpu::new();
		cpu.write_register(register::V0, 1);
		cpu.write_register(register::V0, 2);
		assert_eq!(cpu.read_register(register::V0), 2);
	}

	#[test]
	fn pc_is_mutable() {
		let mut cpu = Cpu::new();
		cpu.pc = 0x0040_0000;
		assert_eq!(cpu.pc, 0x0040_0000);
	}

	#[test]
	fn display_contains_register_names() {
		let cpu = Cpu::new();
		let output = format!("{cpu}");
		assert!(output.contains("zero"));
		assert!(output.contains("ra"));
		assert!(output.contains("CPU State:"));
	}
}
