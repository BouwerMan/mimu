use mimu::asm::assemble;
use mimu::emulator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let path = std::env::args().nth(1).ok_or("usage: emu <file.asm>")?;
	let img = assemble(&path)?;

	let mut emu = emulator::Emulator::new();
	emu.load(&img);
	emu.run()?;

	Ok(())
}
