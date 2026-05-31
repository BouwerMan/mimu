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

// Ex: li $t0, 12
pub fn parse_line(input: &str) -> Result<Parsed, ParseError> {
	let input = input.trim();
	let (mnemonic, rest) = input.split_once(char::is_whitespace).unwrap_or((input, ""));
	let mut args = rest.split(',').map(str::trim).filter(|s| !s.is_empty());

	match mnemonic {
		"li" => {
			let (Some(rd), Some(imm), None) = (args.next(), args.next(), args.next()) else {
				return Err(ParseError::InvalidArgument);
			};
			let rd = parse_register(rd)?;
			let imm = parse_immediate(imm)?;
			Ok(Parsed::Ready(Instruction::Addi {
				rt: rd,
				rs: register::ZERO,
				imm: imm as i16,
			}))
		}
		"add" => {
			let (Some(rd), Some(rs), Some(rt), None) =
				(args.next(), args.next(), args.next(), args.next())
			else {
				return Err(ParseError::InvalidArgument);
			};
			let rd = parse_register(rd)?;
			let rs = parse_register(rs)?;
			let rt = parse_register(rt)?;
			Ok(Parsed::Ready(Instruction::Add { rd, rs, rt }))
		}
		"addi" => {
			let (Some(rt), Some(rs), Some(imm), None) =
				(args.next(), args.next(), args.next(), args.next())
			else {
				return Err(ParseError::InvalidArgument);
			};
			let rt = parse_register(rt)?;
			let rs = parse_register(rs)?;
			let imm = parse_immediate(imm)?;
			Ok(Parsed::Ready(Instruction::Addi {
				rs,
				rt,
				imm: imm as i16,
			}))
		}
		"syscall" => {
			if args.next().is_some() {
				return Err(ParseError::InvalidArgument);
			}
			Ok(Parsed::Ready(Instruction::Syscall))
		}

		"beq" | "bne" => {
			let (Some(rs), Some(rt), Some(label), None) =
				(args.next(), args.next(), args.next(), args.next())
			else {
				return Err(ParseError::InvalidArgument);
			};

			let kind = if mnemonic == "beq" {
				Cond::Eq
			} else {
				Cond::Ne
			};

			Ok(Parsed::Branch {
				kind,
				rs: parse_register(rs)?,
				rt: parse_register(rt)?,
				label: label.to_string(),
			})
		}

		"j" => {
			let (Some(label), None) = (args.next(), args.next()) else {
				return Err(ParseError::InvalidArgument);
			};
			Ok(Parsed::Jump {
				label: label.to_string(),
			})
		}

		_ => Err(ParseError::UnknownInstruction),
	}
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
	use crate::register::{T0, T1, T2};

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

	#[test]
	fn parses_add() {
		assert_eq!(
			parse_line("add $t0, $t1, $t2"),
			Ok(Parsed::Ready(Instruction::Add {
				rd: T0,
				rs: T1,
				rt: T2
			})),
		);
	}

	#[test]
	fn parses_addi() {
		assert_eq!(
			parse_line("addi $t0, $t1, 42"),
			Ok(Parsed::Ready(Instruction::Addi {
				rs: T1,
				rt: T0,
				imm: 42
			})),
		);
	}

	#[test]
	fn parses_syscall() {
		assert_eq!(
			parse_line("syscall"),
			Ok(Parsed::Ready(Instruction::Syscall)),
		);
	}
}
