mod parser;
use crate::instruction;
use std::collections::HashMap;
use thiserror::Error;

pub const TEXT_START: u32 = 0x0040_0000;
pub const DATA_START: u32 = 0x1000_0000;

#[derive(Error, Debug, PartialEq)]
pub enum AsmError {
	#[error("Assembly failed with {errors} error(s).")]
	AsmErrors { errors: usize },
	#[error("Failed to read file {file:?}.")]
	CouldNotReadFile { file: String },
	#[error("Malformed Lines Found")]
	MalformedLines,
	#[error("Duplicate label: {label:?}")]
	DuplicateLabel { label: String },
}

pub struct Image {
	pub text: Vec<u32>,
	pub text_start: u32,
	pub data: Vec<u8>,
	pub data_start: u32,
	pub entry: u32,
}

impl Image {
	fn new() -> Self {
		Self {
			text: Vec::new(),
			text_start: TEXT_START,
			data: Vec::new(),
			data_start: DATA_START,
			entry: TEXT_START,
		}
	}
}

pub fn assemble(path: &str) -> Result<Image, AsmError> {
	let src = match std::fs::read_to_string(path) {
		Ok(s) => s,
		Err(e) => {
			eprintln!("Error: {e}");
			return Err(AsmError::CouldNotReadFile {
				file: path.to_string(),
			});
		}
	};
	assemble_source(&src)
}

pub fn assemble_source(src: &str) -> Result<Image, AsmError> {
	let labels = build_symbol_table(src)?;
	let mut img = Image::new();
	let mut addr = TEXT_START;
	let mut errors = 0usize;

	for (i, line) in src.lines().enumerate() {
		let (_, rest) = split_label(clean(line));
		if rest.is_empty() {
			continue;
		}

		match parser::parse_line(rest).and_then(|p| parser::resolve(p, addr, &labels)) {
			Ok(inst) => img.text.push(instruction::encode(&inst)),
			Err(e) => {
				eprintln!("line {}: error parsing '{rest}': {e}", i + 1);
				errors += 1;
			}
		}
		addr += 4;
	}

	if errors > 0 {
		return Err(AsmError::AsmErrors { errors });
	}

	Ok(img)
}

fn clean(line: &str) -> &str {
	line.split('#').next().unwrap_or("").trim()
}

fn split_label(line: &str) -> (Option<&str>, &str) {
	match line.split_once(':') {
		Some((label, rest)) => (Some(label.trim()), rest.trim()),
		None => (None, line),
	}
}

fn build_symbol_table(src: &str) -> Result<HashMap<String, u32>, AsmError> {
	let mut labels = HashMap::new();
	let mut addr = TEXT_START;

	for line in src.lines() {
		let (label, rest) = split_label(line);
		if let Some(label) = label {
			let key = label.to_string();
			if labels.contains_key(&key) {
				return Err(AsmError::DuplicateLabel { label: key });
			}
			labels.insert(key, addr);
		}
		if !rest.is_empty() {
			addr += 4;
		}
	}

	Ok(labels)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::asm::parser::*;
	use crate::instruction::*;
	use crate::register::*;

	#[test]
	fn assemble_rejects_file_with_bad_line() {
		let src = "li $t0, 1\nadd $t0, $t1\nsyscall\n"; // line 2 is malformed
		assert!(assemble_source(src).is_err());
	}

	#[test]
	fn backward_branch_offset() {
		let labels = HashMap::from([("loop".to_string(), 0x0040_0000)]);
		let inst = parser::resolve(
			Parsed::Branch {
				kind: Cond::Eq,
				rs: T0,
				rt: T1,
				label: "loop".into(),
			},
			0x0040_0008,
			&labels,
		)
		.unwrap();
		assert_eq!(
			inst,
			Instruction::Beq {
				rs: T0,
				rt: T1,
				offset: -3
			}
		); // (−12) >> 2
	}

	#[test]
	fn assembles_loop_with_label() {
		let src = "start: addi $t0, $zero, 3\n\
                   beq $t0, $zero, end\n\
                   addi $t0, $t0, -1\n\
                   j start\n\
                   end: syscall\n";
		let img = assemble_source(src).unwrap();
		assert_eq!(img.text.len(), 5);
		// beq at 0x400004, end at 0x400010 -> offset 2
		assert_eq!(
			instruction::decode(img.text[1]),
			Instruction::Beq {
				rs: T0,
				rt: ZERO,
				offset: 2
			}
		);
	}
}
