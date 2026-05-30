use crate::instruction;
use crate::parser;

pub struct Image {
	pub text: Vec<u32>,
	pub data: Vec<u8>,
	pub entry: u32,
}

impl Image {
	fn new() -> Self {
		Self {
			text: Vec::new(),
			data: Vec::new(),
			entry: 0,
		}
	}
}

pub fn assemble(path: &str) -> Result<Image, Box<dyn std::error::Error>> {
	let src = std::fs::read_to_string(path)?;

	let mut img = Image::new();

	for line in src.lines() {
		let first_char: Vec<char> = line.chars().take(1).collect();
		if first_char.is_empty() || first_char[0].is_whitespace() || first_char[0] == '#' {
			continue; // skip blank lines and comments
		}
		let inst =
			parser::parse_line(line).map_err(|e| format!("Error parsing line '{line}': {e}"))?;
		let inst_raw = instruction::encode(&inst);
		img.text.push(inst_raw);
	}

	Ok(img)
}
