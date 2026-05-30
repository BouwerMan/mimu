mod assembler;
mod cpu;
mod instruction;
mod parser;
mod register;

use assembler::assemble;
use cpu::Cpu;
use instruction::Instruction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let path = std::env::args().nth(1).ok_or("usage: emu <file.asm>")?;
	let img = assemble(&path)?;

	let mut emu = cpu::Emulator::new();
	emu.load(&img);
	emu.run()?;

	Ok(())
}
