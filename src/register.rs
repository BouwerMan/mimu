//! Canonical mapping between MIPS register names and their indices.
//! Single source of truth for parsing, display, and tests.

// The full register file is defined here even though only a few are
// referenced so far; drop this once the rest get used by instructions.
#![allow(dead_code)]

/// Register names indexed by register number. Used for display and,
/// via `name_to_index`, for parsing — so the name→index direction is
/// derived from this rather than duplicated.
pub const NAMES: [&str; 32] = [
	"zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5", "t6",
	"t7", "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "t8", "t9", "k0", "k1", "gp", "sp", "fp",
	"ra",
];

// Human-friendly aliases for referring to a specific register in code.
pub const ZERO: usize = 0;
pub const AT: usize = 1;
pub const V0: usize = 2;
pub const V1: usize = 3;
pub const A0: usize = 4;
pub const A1: usize = 5;
pub const A2: usize = 6;
pub const A3: usize = 7;
pub const T0: usize = 8;
pub const T1: usize = 9;
pub const T2: usize = 10;
pub const T3: usize = 11;
pub const T4: usize = 12;
pub const T5: usize = 13;
pub const T6: usize = 14;
pub const T7: usize = 15;
pub const S0: usize = 16;
pub const S1: usize = 17;
pub const S2: usize = 18;
pub const S3: usize = 19;
pub const S4: usize = 20;
pub const S5: usize = 21;
pub const S6: usize = 22;
pub const S7: usize = 23;
pub const T8: usize = 24;
pub const T9: usize = 25;
pub const K0: usize = 26;
pub const K1: usize = 27;
pub const GP: usize = 28;
pub const SP: usize = 29;
pub const FP: usize = 30;
pub const RA: usize = 31;

/// Look up a register index by name (leading `$` already stripped),
/// accepting the `s8` alias for `fp` (30).
pub fn name_to_index(name: &str) -> Option<usize> {
	if name == "s8" {
		return Some(FP);
	}
	NAMES.iter().position(|&r| r == name)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn names_match_indices() {
		// Guards against the alias constants drifting out of sync with NAMES.
		assert_eq!(NAMES[ZERO], "zero");
		assert_eq!(NAMES[T0], "t0");
		assert_eq!(NAMES[SP], "sp");
		assert_eq!(NAMES[FP], "fp");
		assert_eq!(NAMES[RA], "ra");
	}

	#[test]
	fn s8_aliases_fp() {
		assert_eq!(name_to_index("s8"), Some(FP));
		assert_eq!(name_to_index("fp"), Some(FP));
	}

	#[test]
	fn unknown_name_is_none() {
		assert_eq!(name_to_index("nope"), None);
	}
}
