mod parser;
use crate::instruction;

pub const TEXT_START: u32 = 0x0040_0000;
pub const DATA_START: u32 = 0x1000_0000;

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

pub fn assemble(path: &str) -> Result<Image, Box<dyn std::error::Error>> {
	let src = std::fs::read_to_string(path)?;
	assemble_source(&src)
}

pub fn assemble_source(src: &str) -> Result<Image, Box<dyn std::error::Error>> {
	let mut img = Image::new();
	let mut errors = 0usize;

	for (i, line) in src.lines().enumerate() {
		let line_num = i + 1;
		let line = line.trim();
		if line.is_empty() || line.starts_with('#') {
			continue;
		}
		match parser::parse_line(line) {
			Ok(inst) => {
				let inst_raw = instruction::encode(&inst);
				img.text.push(inst_raw);
			}
			Err(e) => {
				eprintln!("line {line_num}: error parsing '{line}': {e}");
				errors += 1;
			}
		}
	}

	if errors > 0 {
		return Err(format!("assembly failed with {errors} error(s)").into());
	}

	Ok(img)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn assemble_rejects_file_with_bad_line() {
		let src = "li $t0, 1\nadd $t0, $t1\nsyscall\n"; // line 2 is malformed
		assert!(assemble_source(src).is_err());
	}
}
