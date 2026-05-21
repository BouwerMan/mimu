mod cpu;
mod instruction;
mod parser;

use cpu::Cpu;
use instruction::Instruction;

fn main() {
	println!("Hello, world!");
	let mut cpu = Cpu::new();

	println!("CPU state: {cpu:?}");
	cpu.execute(Instruction::LoadImmediate { rd: 9, imm: 16 });
	println!("CPU state: {cpu:?}");

	let inst = parser::parse_line("li $s0, 122").unwrap();
	cpu.execute(inst);
	println!("CPU state: {cpu:?}");
}
