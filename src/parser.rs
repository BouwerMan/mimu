use crate::instruction::Instruction;
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

	match reg.parse::<usize>() {
		Ok(n) if n < 32 => return Ok(n),
		Ok(_) => return Err(ParseError::InvalidArgument), // EX: $40
		Err(_) => {}                                      // not numeric
	}

	match reg {
		"zero" => Ok(0),
		"at" => Ok(1),
		"v0" => Ok(2),
		"v1" => Ok(3),
		"a0" => Ok(4),
		"a1" => Ok(5),
		"a2" => Ok(6),
		"a3" => Ok(7),
		"t0" => Ok(8),
		"t1" => Ok(9),
		"t2" => Ok(10),
		"t3" => Ok(11),
		"t4" => Ok(12),
		"t5" => Ok(13),
		"t6" => Ok(14),
		"t7" => Ok(15),
		"s0" => Ok(16),
		"s1" => Ok(17),
		"s2" => Ok(18),
		"s3" => Ok(19),
		"s4" => Ok(20),
		"s5" => Ok(21),
		"s6" => Ok(22),
		"s7" => Ok(23),
		"t8" => Ok(24),
		"t9" => Ok(25),
		"k0" => Ok(26),
		"k1" => Ok(27),
		"gp" => Ok(28),
		"sp" => Ok(29),
		"fp" | "s8" => Ok(30),
		"ra" => Ok(31),
		_ => Err(ParseError::InvalidArgument),
	}
}

fn parse_immediate(imm: &str) -> Result<i32, ParseError> {
	imm.trim()
		.parse::<i32>()
		.map_err(|_| ParseError::InvalidArgument)
}

// Ex: li $t0, 12
pub fn parse_line(input: &str) -> Result<Instruction, ParseError> {
	let Some((mnemonic, rest)) = input.split_once(char::is_whitespace) else {
		return Err(ParseError::InvalidFormat);
	};

	let mut args = rest.split(',').map(str::trim);

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
		_ => Err(ParseError::UnknownInstruction),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parses_load_immediate() {
		assert_eq!(
			parse_line("li $t0, 42"),
			Ok(Instruction::LoadImmediate { rd: 8, imm: 42 }),
		);
	}

	#[test]
	fn parses_add() {
		assert_eq!(
			parse_line("add $t0, $t1, $t2"),
			Ok(Instruction::Add {
				rd: 8,
				rs: 9,
				rt: 10
			}),
		);
	}
}
