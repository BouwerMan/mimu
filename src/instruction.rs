#[derive(Debug, PartialEq)]
pub enum Instruction {
	Add { rd: usize, rs: usize, rt: usize },
	AddImmediate { rt: usize, rs: usize, imm: i16 },

	Beq { rs: usize, rt: usize, offset: i16 },
	Bne { rs: usize, rt: usize, offset: i16 },
	Jump { target: u32 }, // absolute byte address

	Syscall,
}

pub fn decode(word: u32) -> Instruction {
	use Instruction::*;
	match opcode(word) {
		// SPECIAL: sub-dispatch on funct
		0x00 => match funct(word) {
			0x20 => Add {
				rd: rd(word),
				rs: rs(word),
				rt: rt(word),
			},
			0x0c => Syscall,
			f => unimplemented!("SPECIAL funct {f:#04x}"),
		},
		// // REGIMM: sub-dispatch on the rt field
		// 0x01 => match rt(word) {
		// 	s => unimplemented!("REGIMM rt {s:#04x}"),
		// },
		// // SPECIAL2: sub-dispatch on funct
		// 0x1c => match funct(word) {
		// 	f => unimplemented!("SPECIAL2 funct {f:#04x}"),
		// },
		// J-type: opcode alone
		0x02 => Jump {
			target: target(word) << 2,
		}, // top 4 bits unrecoverable w/o PC; fine in one region

		// I-type: opcode alone
		0x08 => AddImmediate {
			rt: rt(word),
			rs: rs(word),
			imm: imm(word),
		},
		0x04 => Beq {
			rs: rs(word),
			rt: rt(word),
			offset: imm(word),
		},
		0x05 => Bne {
			rs: rs(word),
			rt: rt(word),
			offset: imm(word),
		},

		o => unimplemented!("opcode {o:#04x}"),
	}
}

pub fn encode(inst: &Instruction) -> u32 {
	match inst {
		Instruction::Add { rd, rs, rt } => r_type(0x20, *rs, *rt, *rd, 0),
		Instruction::AddImmediate { rt, rs, imm } => i_type(0x08, *rs, *rt, *imm),

		Instruction::Beq { rs, rt, offset } => i_type(0x04, *rs, *rt, *offset),
		Instruction::Bne { rs, rt, offset } => i_type(0x05, *rs, *rt, *offset),
		Instruction::Jump { target } => j_type(0x02, target >> 2),

		Instruction::Syscall => r_type(0x0c, 0, 0, 0, 0),
		i => unimplemented!("No encoding implemented for instruction {i:?}"),
	}
}

// Decode field accessors
fn opcode(w: u32) -> u32 {
	w >> 26
}
fn rs(w: u32) -> usize {
	((w >> 21) & 0x1f) as usize
}
fn rt(w: u32) -> usize {
	((w >> 16) & 0x1f) as usize
}
fn rd(w: u32) -> usize {
	((w >> 11) & 0x1f) as usize
}
fn shamt(w: u32) -> u32 {
	(w >> 6) & 0x1f
}
fn funct(w: u32) -> u32 {
	w & 0x3f
}
fn imm(w: u32) -> i16 {
	(w & 0xffff) as i16
} // sign-extends
fn target(w: u32) -> u32 {
	w & 0x03ff_ffff
} // raw 26-bit field

// encode helpers
fn r_type(funct: u32, rs: usize, rt: usize, rd: usize, shamt: u32) -> u32 {
	(rs as u32 & 0x1f) << 21
		| (rt as u32 & 0x1f) << 16
		| (rd as u32 & 0x1f) << 11
		| (shamt & 0x1f) << 6
		| (funct & 0x3f)
}

fn i_type(opcode: u32, rs: usize, rt: usize, imm: i16) -> u32 {
	((opcode & 0x3f) << 26)
		| ((rs as u32 & 0x1f) << 21)
		| ((rt as u32 & 0x1f) << 16)
		| (imm as u16 as u32)
}

fn j_type(opcode: u32, target: u32) -> u32 {
	((opcode & 0x3f) << 26) | (target & 0x03ff_ffff)
}

fn regimm(rs: usize, subop: u32, imm: i16) -> u32 {
	(0x01 << 26) | (rs as u32 & 0x1f) << 21 | (subop & 0x1f) << 16 | (imm as u16 as u32)
}

#[cfg(test)]
mod tests {
	use super::*;

	// --- field accessor tests ---

	#[test]
	fn opcode_extracts_top_6_bits() {
		assert_eq!(opcode(0x0800_0000), 0x02);
		assert_eq!(opcode(0x2000_0000), 0x08);
		assert_eq!(opcode(0x0000_0000), 0x00);
		assert_eq!(opcode(0xFC00_0000), 0x3F);
	}

	#[test]
	fn rs_rt_rd_extract_correct_fields() {
		// rs = 8, rt = 9, rd = 10
		let word: u32 = (8 << 21) | (9 << 16) | (10 << 11);
		assert_eq!(rs(word), 8);
		assert_eq!(rt(word), 9);
		assert_eq!(rd(word), 10);
	}

	#[test]
	fn funct_extracts_low_6_bits() {
		assert_eq!(funct(0x0000_0020), 0x20);
		assert_eq!(funct(0x0000_000C), 0x0C);
		assert_eq!(funct(0xFFFF_FFC0), 0x00);
	}

	#[test]
	fn imm_sign_extends_negative() {
		// 0x8000 == -32768 as i16
		assert_eq!(imm(0x0000_8000), -32768i16);
	}

	#[test]
	fn imm_positive_value() {
		assert_eq!(imm(0x0000_7FFF), 0x7FFF);
	}

	#[test]
	fn shamt_extracts_bits_10_to_6() {
		let word: u32 = 0b11111 << 6;
		assert_eq!(shamt(word), 0x1f);
		assert_eq!(shamt(0), 0);
	}

	#[test]
	fn target_extracts_26_bits() {
		assert_eq!(target(0x03FF_FFFF), 0x03FF_FFFF);
		assert_eq!(target(0xFC00_0000), 0);
	}

	// --- encode helpers ---

	#[test]
	fn r_type_packs_fields() {
		let word = r_type(0x20, 8, 9, 10, 0);
		assert_eq!(rs(word), 8);
		assert_eq!(rt(word), 9);
		assert_eq!(rd(word), 10);
		assert_eq!(funct(word), 0x20);
		assert_eq!(opcode(word), 0x00);
	}

	#[test]
	fn i_type_packs_fields() {
		let word = i_type(0x08, 4, 5, -1i16);
		assert_eq!(opcode(word), 0x08);
		assert_eq!(rs(word), 4);
		assert_eq!(rt(word), 5);
		assert_eq!(imm(word), -1i16);
	}

	#[test]
	fn r_type_masks_overflow() {
		// Passing values wider than the field width should be masked cleanly.
		let word = r_type(0x20, 0x3F, 0x3F, 0x3F, 0x3F);
		assert_eq!(rs(word), 0x1F);
		assert_eq!(rt(word), 0x1F);
		assert_eq!(rd(word), 0x1F);
	}

	// --- decode / encode round-trips ---

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

	#[test]
	fn encode_decode_add_immediate() {
		let inst = Instruction::AddImmediate {
			rt: 3,
			rs: 5,
			imm: -100,
		};
		assert_eq!(decode(encode(&inst)), inst);
	}

	#[test]
	fn encode_decode_syscall() {
		assert_eq!(decode(encode(&Instruction::Syscall)), Instruction::Syscall);
	}
}
