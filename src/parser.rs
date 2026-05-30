use crate::instruction::Instruction;
use crate::register;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
	#[error("Invalid instruction format")]
	InvalidFormat,
	#[error("Unknown instruction")]
	UnknownInstruction,
	#[error("Invalid argument")]
	InvalidArgument,
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
pub fn parse_line(input: &str) -> Result<Instruction, ParseError> {
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
			Ok(Instruction::LoadImmediate { rd, imm })
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
			Ok(Instruction::Add { rd, rs, rt })
		}
		"addi" => {
			let (Some(rs), Some(rt), Some(imm), None) =
				(args.next(), args.next(), args.next(), args.next())
			else {
				return Err(ParseError::InvalidArgument);
			};
			let rs = parse_register(rs)?;
			let rt = parse_register(rt)?;
			let imm = parse_immediate(imm)?;
			Ok(Instruction::Addi {
				rs,
				rt,
				imm: imm as i16,
			})
		}
		"syscall" => {
			if args.next().is_some() {
				return Err(ParseError::InvalidArgument);
			}
			Ok(Instruction::Syscall)
		}
		_ => Err(ParseError::UnknownInstruction),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::register::{T0, T1, T2};

	#[test]
	fn parses_load_immediate() {
		assert_eq!(
			parse_line("li $t0, 42"),
			Ok(Instruction::LoadImmediate { rd: T0, imm: 42 }),
		);
	}

	#[test]
	fn parses_add() {
		assert_eq!(
			parse_line("add $t0, $t1, $t2"),
			Ok(Instruction::Add {
				rd: T0,
				rs: T1,
				rt: T2
			}),
		);
	}
}
