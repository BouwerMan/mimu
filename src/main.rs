mod cpu;
mod instruction;
mod parser;

use cpu::Cpu;
use instruction::Instruction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let path = std::env::args().nth(1).ok_or("usage: emu <file.asm>")?;
	let src = std::fs::read_to_string(&path)?;

	let mut cpu = Cpu::new();
	for line in src.lines() {
		let first_char: Vec<char> = line.chars().take(1).collect();
		if first_char.is_empty() || first_char[0].is_whitespace() || first_char[0] == '#' {
			continue; // skip blank lines and comments
		}
		let inst =
			parser::parse_line(line).map_err(|e| format!("Error parsing line '{line}': {e}"))?;
		println!("Executing line: '{line}'");
		cpu.execute(inst);
	}
	println!("{cpu}");
	Ok(())
}
