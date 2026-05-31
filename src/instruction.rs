/// Opcode field (bits 31..26).
mod op {
	pub const SPECIAL: u32 = 0x00; // R-type; dispatch on funct
	pub const REGIMM: u32 = 0x01; // dispatch on rt
	pub const SPECIAL2: u32 = 0x1c; // dispatch on funct
	pub const J: u32 = 0x02;
	pub const JAL: u32 = 0x03;
	pub const BEQ: u32 = 0x04;
	pub const BNE: u32 = 0x05;
	pub const ADDI: u32 = 0x08;
	pub const ADDIU: u32 = 0x09;
	pub const SLTI: u32 = 0x0a;
	pub const SLTIU: u32 = 0x0b;
	pub const ANDI: u32 = 0x0c;
	pub const ORI: u32 = 0x0d;
	pub const XORI: u32 = 0x0e;
	pub const LUI: u32 = 0x0f;
	pub const LW: u32 = 0x23;
	pub const SW: u32 = 0x2b;
}

/// SPECIAL funct field (bits 5..0), valid when opcode == SPECIAL.
mod funct {
	pub const SLL: u32 = 0x00;
	pub const SRL: u32 = 0x02;
	pub const SRA: u32 = 0x03;
	pub const SLLV: u32 = 0x04;
	pub const SRLV: u32 = 0x06;
	pub const SRAV: u32 = 0x07;
	pub const JR: u32 = 0x08;
	pub const JALR: u32 = 0x09;
	pub const SYSCALL: u32 = 0x0c;
	pub const MULT: u32 = 0x18;
	pub const MULTU: u32 = 0x19;
	pub const DIV: u32 = 0x1a;
	pub const DIVU: u32 = 0x1b;
	pub const ADD: u32 = 0x20;
	pub const ADDU: u32 = 0x21;
	pub const SUB: u32 = 0x22;
	pub const SUBU: u32 = 0x23;
	pub const AND: u32 = 0x24;
	pub const OR: u32 = 0x25;
	pub const XOR: u32 = 0x26;
	pub const NOR: u32 = 0x27;
	pub const SLT: u32 = 0x2a;
	pub const SLTU: u32 = 0x2b;
}

// https://student.cs.uwaterloo.ca/~isg/res/mips/opcodes
#[derive(Debug, PartialEq)]
pub enum Instruction {
	Add { rd: usize, rs: usize, rt: usize },
	Addu { rd: usize, rs: usize, rt: usize },
	Addi { rt: usize, rs: usize, imm: i16 },
	Addiu { rt: usize, rs: usize, imm: i16 },

	And { rd: usize, rs: usize, rt: usize },
	Andi { rt: usize, rs: usize, imm: i16 },

	Div { rs: usize, rt: usize },
	Divu { rs: usize, rt: usize },

	Mult { rs: usize, rt: usize },
	Multu { rs: usize, rt: usize },

	Nor { rd: usize, rs: usize, rt: usize },
	Or { rd: usize, rs: usize, rt: usize },
	Ori { rt: usize, rs: usize, imm: i16 },

	Sll { rd: usize, rt: usize, shamt: u32 },
	Sllv { rd: usize, rt: usize, rs: usize },

	Sra { rd: usize, rt: usize, shamt: u32 },
	Srav { rd: usize, rt: usize, rs: usize },
	Srl { rd: usize, rt: usize, shamt: u32 },
	Srlv { rd: usize, rt: usize, rs: usize },

	Sub { rd: usize, rs: usize, rt: usize },
	Subu { rd: usize, rs: usize, rt: usize },

	Xor { rd: usize, rs: usize, rt: usize },
	Xori { rt: usize, rs: usize, imm: i16 },

	Beq { rs: usize, rt: usize, offset: i16 },
	Bne { rs: usize, rt: usize, offset: i16 },
	Jump { target: u32 }, // absolute byte address

	Syscall,
}

pub fn decode(word: u32) -> Instruction {
	use Instruction::*;
	use op::*;
	let (rs, rt, rd, imm) = (rs(word), rt(word), rd(word), imm(word));
	match opcode(word) {
		// SPECIAL: sub-dispatch on funct
		SPECIAL => decode_r_type(word),
		// REGIMM: sub-dispatch on the rt field
		REGIMM => decode_regimm(word),
		// SPECIAL2: sub-dispatch on funct
		SPECIAL2 => decode_special2(word),
		// J-type: opcode alone
		J => Jump {
			target: target(word) << 2,
		}, // top 4 bits unrecoverable w/o PC; fine in one region

		// I-type: opcode alone
		ADDI => Addi { rt, rs, imm },
		ADDIU => Addiu { rt, rs, imm },
		ANDI => Andi { rt, rs, imm },
		ORI => Ori { rt, rs, imm },
		XORI => Xori { rt, rs, imm },
		BEQ => Beq {
			rs,
			rt,
			offset: imm,
		},
		BNE => Bne {
			rs,
			rt,
			offset: imm,
		},

		o => unimplemented!("opcode {o:#04x}"),
	}
}

pub fn encode(inst: &Instruction) -> u32 {
	use Instruction::*;
	use funct::*;
	use op::*;
	match inst {
		Add { rd, rs, rt } => r_type(ADD, *rs, *rt, *rd, 0),
		Addu { rd, rs, rt } => r_type(ADDU, *rs, *rt, *rd, 0),
		Addi { rt, rs, imm } => i_type(0x08, *rs, *rt, *imm),
		Addiu { rt, rs, imm } => i_type(0x09, *rs, *rt, *imm),
		Andi { rt, rs, imm } => i_type(0x0c, *rs, *rt, *imm),

		And { rd, rs, rt } => r_type(AND, *rs, *rt, *rd, 0),

		Div { rs, rt } => r_type(DIV, *rs, *rt, 0, 0),
		Divu { rs, rt } => r_type(DIVU, *rs, *rt, 0, 0),

		Mult { rs, rt } => r_type(MULT, *rs, *rt, 0, 0),
		Multu { rs, rt } => r_type(MULTU, *rs, *rt, 0, 0),

		Nor { rd, rs, rt } => r_type(NOR, *rs, *rt, *rd, 0),
		Or { rd, rs, rt } => r_type(OR, *rs, *rt, *rd, 0),
		Ori { rt, rs, imm } => i_type(ORI, *rs, *rt, *imm),

		Sll { rd, rt, shamt } => r_type(SLL, 0, *rt, *rd, *shamt),
		Sllv { rd, rt, rs } => r_type(SLLV, *rs, *rt, *rd, 0),

		Sra { rd, rt, shamt } => r_type(SRA, 0, *rt, *rd, *shamt),
		Srav { rd, rt, rs } => r_type(SRAV, *rs, *rt, *rd, 0),
		Srl { rd, rt, shamt } => r_type(SRL, 0, *rt, *rd, *shamt),
		Srlv { rd, rt, rs } => r_type(SRLV, *rs, *rt, *rd, 0),

		Sub { rd, rs, rt } => r_type(SUB, *rs, *rt, *rd, 0),
		Subu { rd, rs, rt } => r_type(SUBU, *rs, *rt, *rd, 0),

		Xor { rd, rs, rt } => r_type(XOR, *rs, *rt, *rd, 0),
		Xori { rt, rs, imm } => i_type(0x0e, *rs, *rt, *imm),

		Beq { rs, rt, offset } => i_type(BEQ, *rs, *rt, *offset),
		Bne { rs, rt, offset } => i_type(BNE, *rs, *rt, *offset),
		Jump { target } => j_type(J, target >> 2),

		Syscall => r_type(SYSCALL, 0, 0, 0, 0),
		i => unimplemented!("No encoding implemented for instruction {i:?}"),
	}
}

// Decode helpers

fn decode_r_type(w: u32) -> Instruction {
	use Instruction::*;
	use funct::*;
	let (rs, rt, rd, shamt) = (rs(w), rt(w), rd(w), shamt(w));
	match funct(w) {
		SLL => Sll { rd, rt, shamt },
		SRL => Srl { rd, rt, shamt },
		SRA => Sra { rd, rt, shamt },
		SLLV => Sllv { rd, rt, rs },
		SRLV => Srlv { rd, rt, rs },
		SRAV => Srav { rd, rt, rs },
		MULT => Mult { rs, rt },
		MULTU => Multu { rs, rt },
		DIV => Div { rs, rt },
		DIVU => Divu { rs, rt },
		ADD => Add { rd, rs, rt },
		ADDU => Addu { rd, rs, rt },
		SUB => Sub { rd, rs, rt },
		SUBU => Subu { rd, rs, rt },
		AND => And { rd, rs, rt },
		OR => Or { rd, rs, rt },
		XOR => Xor { rd, rs, rt },
		NOR => Nor { rd, rs, rt },
		SYSCALL => Syscall,
		f => unimplemented!("SPECIAL funct {f:#04x}"),
	}
}

fn decode_regimm(w: u32) -> Instruction {
	use Instruction::*;
	let (rs, subop) = (rs(w), rt(w));
	match subop {
		s => unimplemented!("REGIMM subop {s:#04x}"),
	}
}

fn decode_special2(w: u32) -> Instruction {
	use Instruction::*;
	let (rs, rt, rd, shamt) = (rs(w), rt(w), rd(w), shamt(w));
	match funct(w) {
		f => unimplemented!("SPECIAL2 funct {f:#04x}"),
	}
}

fn decode_j_type(w: u32) -> Instruction {
	use Instruction::*;
	let target = target(w);
	match opcode(w) {
		o => unimplemented!("J-type opcode {o:#04x}"),
	}
}
fn decode_i_type(w: u32) -> Instruction {
	use Instruction::*;
	let (rs, rt, imm) = (rs(w), rt(w), imm(w));
	match opcode(w) {
		o => unimplemented!("I-type opcode {o:#04x}"),
	}
}

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

	macro_rules! round_trip {
		($inst:expr) => {
			assert_eq!(decode(encode(&$inst)), $inst)
		};
	}

	// --- field accessors ---

	#[test]
	fn opcode_extracts_top_6_bits() {
		assert_eq!(opcode(0x0800_0000), 0x02);
		assert_eq!(opcode(0x2000_0000), 0x08);
		assert_eq!(opcode(0x0000_0000), 0x00);
		assert_eq!(opcode(0xFC00_0000), 0x3F);
	}

	#[test]
	fn rs_rt_rd_extract_correct_fields() {
		let word: u32 = (8 << 21) | (9 << 16) | (10 << 11);
		assert_eq!(rs(word), 8);
		assert_eq!(rt(word), 9);
		assert_eq!(rd(word), 10);
	}

	#[test]
	fn funct_extracts_low_6_bits() {
		assert_eq!(funct(0x0000_0020), 0x20);
		assert_eq!(funct(0xFFFF_FFC0), 0x00);
	}

	#[test]
	fn imm_sign_extends() {
		assert_eq!(imm(0x0000_8000), -32768i16);
		assert_eq!(imm(0x0000_7FFF), 0x7FFF);
	}

	#[test]
	fn shamt_extracts_bits_10_to_6() {
		assert_eq!(shamt(0b11111 << 6), 0x1f);
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
		let word = r_type(0x20, 0x3F, 0x3F, 0x3F, 0x3F);
		assert_eq!(rs(word), 0x1F);
		assert_eq!(rt(word), 0x1F);
		assert_eq!(rd(word), 0x1F);
	}

	// --- R-type arithmetic ---

	#[test]
	fn encode_decode_add() {
		round_trip!(Instruction::Add {
			rd: 10,
			rs: 12,
			rt: 14
		});
	}
	#[test]
	fn encode_decode_addu() {
		round_trip!(Instruction::Addu {
			rd: 1,
			rs: 2,
			rt: 3
		});
	}
	#[test]
	fn encode_decode_sub() {
		round_trip!(Instruction::Sub {
			rd: 4,
			rs: 5,
			rt: 6
		});
	}
	#[test]
	fn encode_decode_subu() {
		round_trip!(Instruction::Subu {
			rd: 7,
			rs: 8,
			rt: 9
		});
	}

	// --- R-type logical ---

	#[test]
	fn encode_decode_and() {
		round_trip!(Instruction::And {
			rd: 1,
			rs: 2,
			rt: 3
		});
	}
	#[test]
	fn encode_decode_or() {
		round_trip!(Instruction::Or {
			rd: 4,
			rs: 5,
			rt: 6
		});
	}
	#[test]
	fn encode_decode_xor() {
		round_trip!(Instruction::Xor {
			rd: 7,
			rs: 8,
			rt: 9
		});
	}
	#[test]
	fn encode_decode_nor() {
		round_trip!(Instruction::Nor {
			rd: 10,
			rs: 11,
			rt: 12
		});
	}

	// --- R-type multiply / divide ---

	#[test]
	fn encode_decode_mult() {
		round_trip!(Instruction::Mult { rs: 4, rt: 5 });
	}
	#[test]
	fn encode_decode_multu() {
		round_trip!(Instruction::Multu { rs: 6, rt: 7 });
	}
	#[test]
	fn encode_decode_div() {
		round_trip!(Instruction::Div { rs: 8, rt: 9 });
	}
	#[test]
	fn encode_decode_divu() {
		round_trip!(Instruction::Divu { rs: 10, rt: 11 });
	}

	// --- shifts ---

	#[test]
	fn encode_decode_sll() {
		round_trip!(Instruction::Sll {
			rd: 1,
			rt: 2,
			shamt: 4
		});
	}
	#[test]
	fn encode_decode_srl() {
		round_trip!(Instruction::Srl {
			rd: 3,
			rt: 4,
			shamt: 8
		});
	}
	#[test]
	fn encode_decode_sra() {
		round_trip!(Instruction::Sra {
			rd: 5,
			rt: 6,
			shamt: 16
		});
	}
	#[test]
	fn encode_decode_sllv() {
		round_trip!(Instruction::Sllv {
			rd: 1,
			rt: 2,
			rs: 3
		});
	}
	#[test]
	fn encode_decode_srlv() {
		round_trip!(Instruction::Srlv {
			rd: 4,
			rt: 5,
			rs: 6
		});
	}
	#[test]
	fn encode_decode_srav() {
		round_trip!(Instruction::Srav {
			rd: 7,
			rt: 8,
			rs: 9
		});
	}

	// --- I-type arithmetic ---

	#[test]
	fn encode_decode_addi() {
		round_trip!(Instruction::Addi {
			rt: 3,
			rs: 5,
			imm: -100
		});
	}
	#[test]
	fn encode_decode_addiu() {
		round_trip!(Instruction::Addiu {
			rt: 4,
			rs: 6,
			imm: 200
		});
	}

	// --- I-type logical ---

	#[test]
	fn encode_decode_andi() {
		round_trip!(Instruction::Andi {
			rt: 1,
			rs: 2,
			imm: 0x00FF
		});
	}
	#[test]
	fn encode_decode_ori() {
		round_trip!(Instruction::Ori {
			rt: 3,
			rs: 4,
			imm: 0x0F0F
		});
	}
	#[test]
	fn encode_decode_xori() {
		round_trip!(Instruction::Xori {
			rt: 5,
			rs: 6,
			imm: 0x1234
		});
	}

	// --- branch ---

	#[test]
	fn encode_decode_beq() {
		round_trip!(Instruction::Beq {
			rs: 1,
			rt: 2,
			offset: 16
		});
	}
	#[test]
	fn encode_decode_bne() {
		round_trip!(Instruction::Bne {
			rs: 3,
			rt: 4,
			offset: -8
		});
	}

	// --- jump ---

	#[test]
	fn encode_decode_jump() {
		// target must fit in 28 bits (top 4 bits are lost without a PC)
		round_trip!(Instruction::Jump {
			target: 0x0000_4000
		});
	}

	// --- syscall ---

	#[test]
	fn encode_decode_syscall() {
		round_trip!(Instruction::Syscall);
	}
}
