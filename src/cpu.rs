use crate::instruction::Instruction;

#[derive(Debug)]
pub struct Cpu {
	registers: [i32; 32],
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
		}
	}
}
