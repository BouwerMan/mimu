#[derive(Debug, PartialEq)]
pub enum Instruction {
	LoadImmediate { rd: usize, imm: i32 },
	Add { rd: usize, rs: usize, rt: usize },
}
