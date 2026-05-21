use crate::instruction::Instruction;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Cpu {
	registers: [i32; 32],
}

impl fmt::Display for Cpu {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		const REG_NAMES: [&str; 32] = [
			"zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5",
			"t6", "t7", "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "t8", "t9", "k0", "k1",
			"gp", "sp", "fp", "ra",
		];

		writeln!(f, "CPU State:")?;
		for (i, val) in self.registers.iter().enumerate() {
			write!(f, "${:<4}: {:>11}", REG_NAMES[i], val)?;
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
		Cpu { registers: [0; 32] }
	}

	fn read_register(&self, reg: usize) -> i32 {
		self.registers[reg]
	}

	fn write_register(&mut self, reg: usize, value: i32) {
		self.registers[reg] = value;
	}

	pub fn execute(&mut self, inst: Instruction) {
		match inst {
			Instruction::LoadImmediate { rd, imm } => self.write_register(rd, imm),
			Instruction::Add { rd, rs, rt } => {
				let rs_val = self.read_register(rs);
				let rt_val = self.read_register(rt);
				let result = rs_val.wrapping_add(rt_val); // Use wrapping_add to handle overflow
				self.write_register(rd, result);
			}
			_ => unimplemented!("Instruction not implemented yet"),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_load_immediate() {
		let mut cpu = Cpu::new();
		cpu.execute(Instruction::LoadImmediate { rd: 8, imm: 42 }); // li $t0, 42
		assert_eq!(cpu.read_register(8), 42);
	}

	#[test]
	fn test_add() {
		let mut cpu = Cpu::new();
		cpu.execute(Instruction::LoadImmediate { rd: 8, imm: 10 }); // li $t0, 10
		cpu.execute(Instruction::LoadImmediate { rd: 9, imm: 20 }); // li $t1, 20
		cpu.execute(Instruction::Add {
			rd: 10,
			rs: 8,
			rt: 9,
		}); // add $t2, $t0, $t1
		assert_eq!(cpu.read_register(10), 30);
	}
}
