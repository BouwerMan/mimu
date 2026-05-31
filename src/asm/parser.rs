use std::collections::HashMap;

use crate::instruction::Instruction;
use crate::register;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub enum Parsed {
	Ready(Instruction), // add, addi, li, syscall…
	Branch {
		kind: Cond,
		rs: usize,
		rt: usize,
		label: String,
	},
	Jump {
		label: String,
	},
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Cond {
	Eq,
	Ne,
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
	#[error("Unknown instruction")]
	UnknownInstruction,
	#[error("Invalid argument")]
	InvalidArgument,
	#[error("Undefined label: {label:?}")]
	UndefinedLabel { label: String },
	#[error(
		"Branch out of range (Here: {here:#04x}, Target: {target:#04x}, delta >> 2: {delta:#04x})"
	)]
	BranchOutOfRange { here: u32, target: u32, delta: i64 },
}

fn parse_register(reg: &str) -> Result<usize, ParseError> {
	let reg = reg
		.trim()
		.strip_prefix('$')
		.ok_or(ParseError::InvalidArgument)?;

	// Numeric form, e.g. `$8`.
	if let Ok(n) = reg.parse::<usize>() {
		return if n < 32 {
			Ok(n)
		} else {
			Err(ParseError::InvalidArgument) // EX: $40
		};
	}

	// Named form, e.g. `$t0` (or the `s8` alias for `$fp`).
	register::name_to_index(reg).ok_or(ParseError::InvalidArgument)
}

fn parse_immediate(imm: &str) -> Result<i32, ParseError> {
	imm.trim()
		.parse::<i32>()
		.map_err(|_| ParseError::InvalidArgument)
}

fn next_reg<'a>(args: &mut impl Iterator<Item = &'a str>) -> Result<usize, ParseError> {
	parse_register(args.next().ok_or(ParseError::InvalidArgument)?)
}

fn next_imm<'a>(args: &mut impl Iterator<Item = &'a str>) -> Result<i32, ParseError> {
	parse_immediate(args.next().ok_or(ParseError::InvalidArgument)?)
}

fn next_label<'a>(args: &mut impl Iterator<Item = &'a str>) -> Result<String, ParseError> {
	Ok(args.next().ok_or(ParseError::InvalidArgument)?.to_string())
}

fn expect_end<'a>(args: &mut impl Iterator<Item = &'a str>) -> Result<(), ParseError> {
	match args.next() {
		None => Ok(()),
		Some(_) => Err(ParseError::InvalidArgument),
	}
}

// Ex: li $t0, 12
pub fn parse_line(input: &str) -> Result<Parsed, ParseError> {
	let input = input.trim();
	let (mnemonic, rest) = input.split_once(char::is_whitespace).unwrap_or((input, ""));
	let mut args = rest.split(',').map(str::trim).filter(|s| !s.is_empty());

	match mnemonic {
		"add" => {
			let (rd, rs, rt) = fmt_r3(&mut args)?;
			Ok(Parsed::Ready(Instruction::Add { rd, rs, rt }))
		}
		"addu" => {
			let (rd, rs, rt) = fmt_r3(&mut args)?;
			Ok(Parsed::Ready(Instruction::Addu { rd, rs, rt }))
		}
		"addi" => {
			let (rt, rs, imm) = fmt_itype(&mut args)?;
			Ok(Parsed::Ready(Instruction::Addi { rs, rt, imm }))
		}
		"addiu" => {
			let (rt, rs, imm) = fmt_itype(&mut args)?;
			Ok(Parsed::Ready(Instruction::Addiu { rs, rt, imm }))
		}

		"and" => {
			let (rd, rs, rt) = fmt_r3(&mut args)?;
			Ok(Parsed::Ready(Instruction::And { rd, rs, rt }))
		}
		"andi" => {
			let (rt, rs, imm) = fmt_itype(&mut args)?;
			Ok(Parsed::Ready(Instruction::Andi { rt, rs, imm }))
		}

		"div" => {
			let (rs, rt) = fmt_r2(&mut args)?;
			Ok(Parsed::Ready(Instruction::Div { rs, rt }))
		}
		"divu" => {
			let (rs, rt) = fmt_r2(&mut args)?;
			Ok(Parsed::Ready(Instruction::Divu { rs, rt }))
		}

		"nor" => {
			let (rd, rs, rt) = fmt_r3(&mut args)?;
			Ok(Parsed::Ready(Instruction::Nor { rd, rs, rt }))
		}
		"or" => {
			let (rd, rs, rt) = fmt_r3(&mut args)?;
			Ok(Parsed::Ready(Instruction::Or { rd, rs, rt }))
		}
		"ori" => {
			let (rt, rs, imm) = fmt_itype(&mut args)?;
			Ok(Parsed::Ready(Instruction::Ori { rt, rs, imm }))
		}

		"sll" => {
			let (rd, rt, shamt) = fmt_shamt(&mut args)?;
			Ok(Parsed::Ready(Instruction::Sll { rd, rt, shamt }))
		}
		"sllv" => {
			let (rd, rt, rs) = fmt_shamt_var(&mut args)?;
			Ok(Parsed::Ready(Instruction::Sllv { rd, rt, rs }))
		}

		"sra" => {
			let (rd, rt, shamt) = fmt_shamt(&mut args)?;
			Ok(Parsed::Ready(Instruction::Sra { rd, rt, shamt }))
		}
		"srav" => {
			let (rd, rt, rs) = fmt_shamt_var(&mut args)?;
			Ok(Parsed::Ready(Instruction::Srav { rd, rt, rs }))
		}
		"srl" => {
			let (rd, rt, shamt) = fmt_shamt(&mut args)?;
			Ok(Parsed::Ready(Instruction::Srl { rd, rt, shamt }))
		}
		"srlv" => {
			let (rd, rt, rs) = fmt_shamt_var(&mut args)?;
			Ok(Parsed::Ready(Instruction::Srlv { rd, rt, rs }))
		}

		"sub" => {
			let (rd, rs, rt) = fmt_r3(&mut args)?;
			Ok(Parsed::Ready(Instruction::Sub { rd, rs, rt }))
		}
		"subu" => {
			let (rd, rs, rt) = fmt_r3(&mut args)?;
			Ok(Parsed::Ready(Instruction::Subu { rd, rs, rt }))
		}

		"xor" => {
			let (rd, rs, rt) = fmt_r3(&mut args)?;
			Ok(Parsed::Ready(Instruction::Xor { rd, rs, rt }))
		}
		"xori" => {
			let (rt, rs, imm) = fmt_itype(&mut args)?;
			Ok(Parsed::Ready(Instruction::Xori { rt, rs, imm }))
		}

		"beq" | "bne" => {
			let (rs, rt, label) = fmt_branch(&mut args)?;
			let kind = if mnemonic == "beq" {
				Cond::Eq
			} else {
				Cond::Ne
			};
			Ok(Parsed::Branch {
				kind,
				rs,
				rt,
				label,
			})
		}
		"j" => {
			let label = next_label(&mut args)?;
			expect_end(&mut args)?;
			Ok(Parsed::Jump { label })
		}

		"syscall" => {
			expect_end(&mut args)?;
			Ok(Parsed::Ready(Instruction::Syscall))
		}

		// Pseudo Instructions
		"li" => {
			let rd = next_reg(&mut args)?;
			let imm = next_imm(&mut args)?;
			expect_end(&mut args)?;
			Ok(Parsed::Ready(Instruction::Addi {
				rt: rd,
				rs: register::ZERO,
				imm: imm as i16,
			}))
		}
		_ => Err(ParseError::UnknownInstruction),
	}
}

fn fmt_r3<'a>(
	args: &mut impl Iterator<Item = &'a str>,
) -> Result<(usize, usize, usize), ParseError> {
	let rd = next_reg(args)?;
	let rs = next_reg(args)?;
	let rt = next_reg(args)?;
	expect_end(args)?;
	Ok((rd, rs, rt))
}

fn fmt_itype<'a>(
	args: &mut impl Iterator<Item = &'a str>,
) -> Result<(usize, usize, i16), ParseError> {
	let rt = next_reg(args)?;
	let rs = next_reg(args)?;
	let imm = next_imm(args)? as i16;
	expect_end(args)?;
	Ok((rt, rs, imm))
}

fn fmt_shamt<'a>(
	args: &mut impl Iterator<Item = &'a str>,
) -> Result<(usize, usize, u32), ParseError> {
	let rd = next_reg(args)?;
	let rt = next_reg(args)?;
	let shamt = next_imm(args)? as u32;
	expect_end(args)?;
	Ok((rd, rt, shamt))
}

fn fmt_shamt_var<'a>(
	args: &mut impl Iterator<Item = &'a str>,
) -> Result<(usize, usize, usize), ParseError> {
	let rd = next_reg(args)?;
	let rt = next_reg(args)?;
	let rs = next_reg(args)?;
	expect_end(args)?;
	Ok((rd, rt, rs))
}

fn fmt_r2<'a>(args: &mut impl Iterator<Item = &'a str>) -> Result<(usize, usize), ParseError> {
	let rd = next_reg(args)?;
	let rs = next_reg(args)?;
	expect_end(args)?;
	Ok((rd, rs))
}

fn fmt_branch<'a>(
	args: &mut impl Iterator<Item = &'a str>,
) -> Result<(usize, usize, String), ParseError> {
	let rs = next_reg(args)?;
	let rt = next_reg(args)?;
	let label = next_label(args)?;
	expect_end(args)?;
	Ok((rs, rt, label))
}

fn branch_offset(here: u32, target: u32) -> Result<i16, ParseError> {
	let delta = target as i64 - (here as i64 + 4);
	i16::try_from(delta >> 2).map_err(|_| ParseError::BranchOutOfRange {
		here,
		target,
		delta: delta >> 2,
	})
}

pub fn resolve(
	parsed: Parsed,
	here: u32,
	labels: &HashMap<String, u32>,
) -> Result<Instruction, ParseError> {
	match parsed {
		Parsed::Ready(inst) => Ok(inst),
		Parsed::Branch {
			kind,
			rs,
			rt,
			label,
		} => {
			let target = *labels
				.get(&label)
				.ok_or(ParseError::UndefinedLabel { label })?;
			let offset = branch_offset(here, target)?;
			Ok(match kind {
				Cond::Eq => Instruction::Beq { rs, rt, offset },
				Cond::Ne => Instruction::Bne { rs, rt, offset },
			})
		}
		Parsed::Jump { label } => {
			let target = *labels
				.get(&label)
				.ok_or(ParseError::UndefinedLabel { label })?;
			Ok(Instruction::Jump { target })
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::register::{self, T0, T1, T2};

	#[test]
	fn malformed_lines_are_rejected() {
		let cases = [
			("foo $t0, $t1, $t2", ParseError::UnknownInstruction),
			("add t0, $t1, $t2", ParseError::InvalidArgument), // missing $
			("add $t0, $t1", ParseError::InvalidArgument),     // too few args
			("add $t0, $t1, $t2, $t3", ParseError::InvalidArgument), // too many
			("addi $t0, $40, 1", ParseError::InvalidArgument), // reg out of range
			("li $nope, 1", ParseError::InvalidArgument),      // bad reg name
			("li $t0, banana", ParseError::InvalidArgument),   // non-numeric imm
			("syscall $t0", ParseError::InvalidArgument),      // args on syscall
		];
		for (src, expected) in cases {
			assert_eq!(parse_line(src), Err(expected), "input: {src:?}");
		}
	}

	// --- arithmetic ---

	#[test]
	fn parses_add() {
		assert_eq!(
			parse_line("add $t0, $t1, $t2"),
			Ok(Parsed::Ready(Instruction::Add { rd: T0, rs: T1, rt: T2 })),
		);
	}

	#[test]
	fn parses_addu() {
		assert_eq!(
			parse_line("addu $t0, $t1, $t2"),
			Ok(Parsed::Ready(Instruction::Addu { rd: T0, rs: T1, rt: T2 })),
		);
	}

	#[test]
	fn parses_addi() {
		assert_eq!(
			parse_line("addi $t0, $t1, 42"),
			Ok(Parsed::Ready(Instruction::Addi { rt: T0, rs: T1, imm: 42 })),
		);
	}

	#[test]
	fn parses_addiu() {
		assert_eq!(
			parse_line("addiu $t0, $t1, -1"),
			Ok(Parsed::Ready(Instruction::Addiu { rt: T0, rs: T1, imm: -1 })),
		);
	}

	#[test]
	fn parses_sub() {
		assert_eq!(
			parse_line("sub $t0, $t1, $t2"),
			Ok(Parsed::Ready(Instruction::Sub { rd: T0, rs: T1, rt: T2 })),
		);
	}

	#[test]
	fn parses_subu() {
		assert_eq!(
			parse_line("subu $t0, $t1, $t2"),
			Ok(Parsed::Ready(Instruction::Subu { rd: T0, rs: T1, rt: T2 })),
		);
	}

	#[test]
	fn parses_div() {
		assert_eq!(
			parse_line("div $t0, $t1"),
			Ok(Parsed::Ready(Instruction::Div { rs: T0, rt: T1 })),
		);
	}

	#[test]
	fn parses_divu() {
		assert_eq!(
			parse_line("divu $t0, $t1"),
			Ok(Parsed::Ready(Instruction::Divu { rs: T0, rt: T1 })),
		);
	}

	// --- logical ---

	#[test]
	fn parses_and() {
		assert_eq!(
			parse_line("and $t0, $t1, $t2"),
			Ok(Parsed::Ready(Instruction::And { rd: T0, rs: T1, rt: T2 })),
		);
	}

	#[test]
	fn parses_andi() {
		assert_eq!(
			parse_line("andi $t0, $t1, 255"),
			Ok(Parsed::Ready(Instruction::Andi { rt: T0, rs: T1, imm: 0xFF })),
		);
	}

	#[test]
	fn parses_or() {
		assert_eq!(
			parse_line("or $t0, $t1, $t2"),
			Ok(Parsed::Ready(Instruction::Or { rd: T0, rs: T1, rt: T2 })),
		);
	}

	#[test]
	fn parses_ori() {
		assert_eq!(
			parse_line("ori $t0, $t1, 7"),
			Ok(Parsed::Ready(Instruction::Ori { rt: T0, rs: T1, imm: 7 })),
		);
	}

	#[test]
	fn parses_nor() {
		assert_eq!(
			parse_line("nor $t0, $t1, $t2"),
			Ok(Parsed::Ready(Instruction::Nor { rd: T0, rs: T1, rt: T2 })),
		);
	}

	#[test]
	fn parses_xor() {
		assert_eq!(
			parse_line("xor $t0, $t1, $t2"),
			Ok(Parsed::Ready(Instruction::Xor { rd: T0, rs: T1, rt: T2 })),
		);
	}

	#[test]
	fn parses_xori() {
		assert_eq!(
			parse_line("xori $t0, $t1, 3"),
			Ok(Parsed::Ready(Instruction::Xori { rt: T0, rs: T1, imm: 3 })),
		);
	}

	// --- shifts ---

	#[test]
	fn parses_sll() {
		assert_eq!(
			parse_line("sll $t0, $t1, 2"),
			Ok(Parsed::Ready(Instruction::Sll { rd: T0, rt: T1, shamt: 2 })),
		);
	}

	#[test]
	fn parses_sllv() {
		assert_eq!(
			parse_line("sllv $t0, $t1, $t2"),
			Ok(Parsed::Ready(Instruction::Sllv { rd: T0, rt: T1, rs: T2 })),
		);
	}

	#[test]
	fn parses_srl() {
		assert_eq!(
			parse_line("srl $t0, $t1, 1"),
			Ok(Parsed::Ready(Instruction::Srl { rd: T0, rt: T1, shamt: 1 })),
		);
	}

	#[test]
	fn parses_srlv() {
		assert_eq!(
			parse_line("srlv $t0, $t1, $t2"),
			Ok(Parsed::Ready(Instruction::Srlv { rd: T0, rt: T1, rs: T2 })),
		);
	}

	#[test]
	fn parses_sra() {
		assert_eq!(
			parse_line("sra $t0, $t1, 3"),
			Ok(Parsed::Ready(Instruction::Sra { rd: T0, rt: T1, shamt: 3 })),
		);
	}

	#[test]
	fn parses_srav() {
		assert_eq!(
			parse_line("srav $t0, $t1, $t2"),
			Ok(Parsed::Ready(Instruction::Srav { rd: T0, rt: T1, rs: T2 })),
		);
	}

	// --- pseudo ---

	#[test]
	fn parses_li() {
		assert_eq!(
			parse_line("li $t0, 5"),
			Ok(Parsed::Ready(Instruction::Addi { rt: T0, rs: register::ZERO, imm: 5 })),
		);
	}

	#[test]
	fn parses_syscall() {
		assert_eq!(parse_line("syscall"), Ok(Parsed::Ready(Instruction::Syscall)));
	}

	// --- control flow ---

	#[test]
	fn parses_beq() {
		assert_eq!(
			parse_line("beq $t0, $t1, loop"),
			Ok(Parsed::Branch { kind: Cond::Eq, rs: T0, rt: T1, label: "loop".into() }),
		);
	}

	#[test]
	fn parses_bne() {
		assert_eq!(
			parse_line("bne $t0, $t1, loop"),
			Ok(Parsed::Branch { kind: Cond::Ne, rs: T0, rt: T1, label: "loop".into() }),
		);
	}

	#[test]
	fn parses_j() {
		assert_eq!(
			parse_line("j exit"),
			Ok(Parsed::Jump { label: "exit".into() }),
		);
	}
}
