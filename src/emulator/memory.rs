use std::collections::HashMap;

#[derive(Debug)]
pub struct Memory {
	data: HashMap<u32, u8>,
}

impl Memory {
	pub fn new() -> Self {
		Memory {
			data: HashMap::new(),
		}
	}

	fn read_byte(&self, addr: u32) -> u8 {
		self.data.get(&addr).copied().unwrap_or(0)
	}

	pub fn read_word(&self, addr: u32) -> u32 {
		let mut word: u32 = 0;
		for byte in 0..4 {
			word |= (self.read_byte(addr + byte) as u32) << (byte * 8);
		}
		word
	}

	fn write_byte(&mut self, addr: u32, data: u8) {
		self.data.insert(addr, data);
	}

	pub fn write_word(&mut self, addr: u32, word: u32) {
		for byte in 0..4 {
			self.write_byte(addr + byte, (word >> (byte * 8)) as u8);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn unwritten_address_reads_zero() {
		let mem = Memory::new();
		assert_eq!(mem.read_word(0x0000_0000), 0);
		assert_eq!(mem.read_word(0xFFFF_FFFC), 0);
	}

	#[test]
	fn write_then_read_word() {
		let mut mem = Memory::new();
		mem.write_word(0x100, 0xDEAD_BEEF);
		assert_eq!(mem.read_word(0x100), 0xDEAD_BEEF);
	}

	#[test]
	fn write_max_u32() {
		let mut mem = Memory::new();
		mem.write_word(0x200, u32::MAX);
		assert_eq!(mem.read_word(0x200), u32::MAX);
	}

	#[test]
	fn little_endian_byte_layout() {
		let mut mem = Memory::new();
		mem.write_word(0x0, 0x0102_0304);
		// byte 0 (lowest addr) holds the least-significant byte
		assert_eq!(mem.read_byte(0x0), 0x04);
		assert_eq!(mem.read_byte(0x1), 0x03);
		assert_eq!(mem.read_byte(0x2), 0x02);
		assert_eq!(mem.read_byte(0x3), 0x01);
	}

	#[test]
	fn overwrite_word() {
		let mut mem = Memory::new();
		mem.write_word(0x0, 0xAAAA_AAAA);
		mem.write_word(0x0, 0x1234_5678);
		assert_eq!(mem.read_word(0x0), 0x1234_5678);
	}

	#[test]
	fn write_does_not_affect_adjacent_words() {
		let mut mem = Memory::new();
		mem.write_word(0x100, 0xCAFE_BABE);
		assert_eq!(mem.read_word(0x0FC), 0);
		assert_eq!(mem.read_word(0x104), 0);
	}

	#[test]
	fn multiple_independent_addresses() {
		let mut mem = Memory::new();
		mem.write_word(0x000, 1);
		mem.write_word(0x100, 2);
		mem.write_word(0x200, 3);
		assert_eq!(mem.read_word(0x000), 1);
		assert_eq!(mem.read_word(0x100), 2);
		assert_eq!(mem.read_word(0x200), 3);
	}

	#[test]
	fn write_zero_clears_word() {
		let mut mem = Memory::new();
		mem.write_word(0x0, 0xFFFF_FFFF);
		mem.write_word(0x0, 0x0000_0000);
		assert_eq!(mem.read_word(0x0), 0);
	}
}
