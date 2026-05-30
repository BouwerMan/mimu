#[derive(Debug, PartialEq)]
pub enum Instruction {
	LoadImmediate { rd: usize, imm: i32 },
	Add { rd: usize, rs: usize, rt: usize },
	Addi { rs: usize, rt: usize, imm: i16 },
	Syscall,
}

pub fn decode(word: u32) -> Instruction {
	let opcode = word >> 26;
	let funct = word & 0x3f; // Low 6 bits
	match (opcode, funct) {
		// R-type
		(0x00, 0x20) => Instruction::Add {
			rd: ((word >> 11) & 0x1f) as usize, // 0x5f = 5-bit field
			rs: ((word >> 21) & 0x1f) as usize,
			rt: ((word >> 16) & 0x1f) as usize,
		},
		(0x00, 0x0c) => Instruction::Syscall,

		// I-type
		(0x08, _) => Instruction::Addi {
			rs: ((word >> 21) & 0x1f) as usize,
			rt: ((word >> 16) & 0x1f) as usize,
			imm: (word & 0xffff) as i16, // imm is low 16 bits
		},
		_ => unimplemented!("opcode/funct not decoded yet"),
	}
}

pub fn encode(inst: &Instruction) -> u32 {
	match inst {
		Instruction::Add { rd, rs, rt } => {
			let opcode = 0x00 << 26;
			let funct = 0x20;
			let mut word = opcode | funct;
			word |= (rd & 0x1f) << 11;
			word |= (rs & 0x1f) << 21;
			word |= (rt & 0x1f) << 16;
			word as u32
		}
		Instruction::Addi { rs, rt, imm } => {
			let opcode = 0x08 << 26;
			let mut word = opcode;
			word |= (rt & 0x1f) << 16;
			word |= (rs & 0x1f) << 21;
			word |= *imm as usize & 0xffff; // imm is 16 bits
			word as u32
		}
		Instruction::Syscall => {
			let opcode = 0x00 << 26;
			let funct = 0x0c;
			(opcode | funct) as u32
		}
		_ => unimplemented!("instruction not encoded yet"),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn encode_decode_add() {
		let inst = Instruction::Add {
			rd: 10,
			rs: 12,
			rt: 14,
		};
		assert_eq!(decode(encode(&inst)), inst);
	}

	#[test]
	fn decode_encode_add() {
		let word = (10 << 21) | (12 << 16) | (14 << 11) | 0x20;
		assert_eq!(encode(&decode(word)), word);
	}
}
