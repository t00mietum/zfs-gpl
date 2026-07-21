// SPDX-License-Identifier: GPL-2.0-or-later
//
// Uberblock parsing and active-uberblock selection. Spec:
// spec/specs/format/01-vdev-label-uberblock.md, sections 5, 7, 8.

//! An uberblock is a packed run of 64-bit fields in the writing host's native
//! byte order; a reader detects the order from the magic and returns every
//! field in host order. Fields past the root pointer were appended over time
//! and never reordered, so an older uberblock parses by its leading fields with
//! the trailing ones reading as zero. (spec 5)

/// Uberblock magic, read as a native u64. (spec 5, 9)
pub const UBERBLOCK_MAGIC: u64 = 0x0000_0000_00ba_b10c;

/// The same magic seen when the writer used the opposite byte order. (spec 7)
pub const UBERBLOCK_MAGIC_BSWAP: u64 = UBERBLOCK_MAGIC.swap_bytes();

/// MMP sub-block magic. (spec 5, 9)
pub const MMP_MAGIC: u64 = 0x0000_0000_a11c_ea11;

// Field offsets within an uberblock slot. (spec 5)
const OFF_MAGIC: usize = 0;
const OFF_VERSION: usize = 8;
const OFF_TXG: usize = 16;
const OFF_GUID_SUM: usize = 24;
const OFF_TIMESTAMP: usize = 32;
const OFF_ROOTBP: usize = 40;
const OFF_SOFTWARE_VERSION: usize = 168;
const OFF_MMP_MAGIC: usize = 176;
const OFF_MMP_DELAY: usize = 184;
const OFF_MMP_CONFIG: usize = 192;
const OFF_CHECKPOINT_TXG: usize = 200;
const OFF_RAIDZ_REFLOW: usize = 208;

/// The 128-byte root block pointer, addressing the Meta-Object-Set. (spec 5, 9)
pub const ROOTBP_SIZE: usize = 128;

/// Size of the fixed portion of an uberblock. (spec 5)
pub const UBERBLOCK_FIXED_SIZE: usize = 216;

/// Byte order the uberblock was written in, relative to the reading host. (spec 7)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteOrder {
	Native,
	Swapped,
}

/// A parsed uberblock, all integer fields in host order. The root block pointer
/// is kept as raw bytes; its internal layout belongs to the block-pointer spec.
#[derive(Clone, Debug)]
pub struct Uberblock {
	pub byte_order: ByteOrder,
	pub version: u64,
	pub txg: u64,
	pub guid_sum: u64,
	pub timestamp: u64,
	pub rootbp: [u8; ROOTBP_SIZE],
	pub software_version: u64,
	pub mmp_magic: u64,
	pub mmp_delay: u64,
	pub mmp_config: u64,
	pub checkpoint_txg: u64,
	pub raidz_reflow_info: u64,
}

impl Uberblock {
	/// Parse an uberblock from the front of a slot. Returns `None` if the slot
	/// is too small or the magic does not identify a valid uberblock. This is
	/// the structural half of the validity gate; the caller pairs it with the
	/// self-checksum check before trusting a candidate. (spec 5, 7, 8.1)
	#[must_use]
	pub fn parse(slot: &[u8]) -> Option<Uberblock> {
		if slot.len() < UBERBLOCK_FIXED_SIZE {
			return None;
		}
		let byte_order = match read_u64_le(slot, OFF_MAGIC) {
			UBERBLOCK_MAGIC => ByteOrder::Native,
			UBERBLOCK_MAGIC_BSWAP => ByteOrder::Swapped,
			_ => return None,
		};
		let field = |offset: usize| -> u64 {
			let raw = read_u64_le(slot, offset);
			match byte_order {
				ByteOrder::Native => raw,
				ByteOrder::Swapped => raw.swap_bytes(),
			}
		};
		let mut rootbp = [0u8; ROOTBP_SIZE];
		rootbp.copy_from_slice(&slot[OFF_ROOTBP..OFF_ROOTBP + ROOTBP_SIZE]);
		Some(Uberblock {
			byte_order,
			version: field(OFF_VERSION),
			txg: field(OFF_TXG),
			guid_sum: field(OFF_GUID_SUM),
			timestamp: field(OFF_TIMESTAMP),
			rootbp,
			software_version: field(OFF_SOFTWARE_VERSION),
			mmp_magic: field(OFF_MMP_MAGIC),
			mmp_delay: field(OFF_MMP_DELAY),
			mmp_config: field(OFF_MMP_CONFIG),
			checkpoint_txg: field(OFF_CHECKPOINT_TXG),
			raidz_reflow_info: field(OFF_RAIDZ_REFLOW),
		})
	}

	/// Whether this uberblock carries valid MMP (multi-host) metadata. (spec 5, 6.4)
	#[must_use]
	pub fn mmp_valid(&self) -> bool {
		self.mmp_magic == MMP_MAGIC
	}

	/// MMP sequence number, if present and flagged valid by `mmp_config`. (spec 6.4)
	#[must_use]
	pub fn mmp_sequence(&self) -> Option<u16> {
		const SEQ_VALID_BIT: u64 = 0x02;
		if !self.mmp_valid() || self.mmp_config & SEQ_VALID_BIT == 0 {
			return None;
		}
		Some(((self.mmp_config >> 32) & 0xffff) as u16)
	}

	/// Whether this uberblock is a saved pool checkpoint. (spec 5, 8.4)
	#[must_use]
	pub fn is_checkpoint(&self) -> bool {
		self.checkpoint_txg != 0
	}
}

/// Choose the active uberblock among already-validated candidates: highest
/// `txg`, then higher `timestamp`, then higher MMP sequence; any remaining tie
/// is acceptable. Returns the winner's index, or `None` for an empty slice.
/// Callers pass only candidates that already cleared the validity gate. (spec 8.2, 8.3)
#[must_use]
pub fn select_active(candidates: &[Uberblock]) -> Option<usize> {
	candidates
		.iter()
		.enumerate()
		.max_by_key(|(_, uberblock)| selection_rank(uberblock))
		.map(|(i, _)| i)
}

/// Ordering key for active-uberblock selection: `(txg, timestamp, mmp sequence)`
/// compared lexicographically, highest wins. Exposed so a scanner ranking its
/// own candidate set applies the same tie-breaks. (spec 8.2, 8.3)
#[must_use]
pub fn selection_rank(uberblock: &Uberblock) -> (u64, u64, u16) {
	(
		uberblock.txg,
		uberblock.timestamp,
		uberblock.mmp_sequence().unwrap_or(0),
	)
}

fn read_u64_le(buf: &[u8], offset: usize) -> u64 {
	let mut bytes = [0u8; 8];
	bytes.copy_from_slice(&buf[offset..offset + 8]);
	u64::from_le_bytes(bytes)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn blank_slot() -> Vec<u8> {
		vec![0u8; 1024]
	}

	#[test]
	fn parses_native_uberblock() {
		let mut slot = blank_slot();
		slot[OFF_MAGIC..OFF_MAGIC + 8].copy_from_slice(&UBERBLOCK_MAGIC.to_le_bytes());
		slot[OFF_TXG..OFF_TXG + 8].copy_from_slice(&42u64.to_le_bytes());
		slot[OFF_TIMESTAMP..OFF_TIMESTAMP + 8].copy_from_slice(&1_000u64.to_le_bytes());
		let uberblock = Uberblock::parse(&slot).unwrap();
		assert_eq!(uberblock.byte_order, ByteOrder::Native);
		assert_eq!(uberblock.txg, 42);
		assert_eq!(uberblock.timestamp, 1_000);
	}

	#[test]
	fn parses_byteswapped_uberblock() {
		// A writer of the opposite endianness stores each field big-endian.
		let mut slot = blank_slot();
		slot[OFF_MAGIC..OFF_MAGIC + 8].copy_from_slice(&UBERBLOCK_MAGIC.to_be_bytes());
		slot[OFF_TXG..OFF_TXG + 8].copy_from_slice(&42u64.to_be_bytes());
		let uberblock = Uberblock::parse(&slot).unwrap();
		assert_eq!(uberblock.byte_order, ByteOrder::Swapped);
		assert_eq!(uberblock.txg, 42);
	}

	#[test]
	fn rejects_garbage_and_short_slots() {
		assert!(Uberblock::parse(&blank_slot()).is_none()); // no magic
		assert!(Uberblock::parse(&[0u8; 8]).is_none()); // too short
	}

	#[test]
	fn active_is_highest_txg_then_timestamp() {
		let mk = |txg, timestamp| Uberblock {
			byte_order: ByteOrder::Native,
			version: 0,
			txg,
			guid_sum: 0,
			timestamp,
			rootbp: [0u8; ROOTBP_SIZE],
			software_version: 0,
			mmp_magic: 0,
			mmp_delay: 0,
			mmp_config: 0,
			checkpoint_txg: 0,
			raidz_reflow_info: 0,
		};
		let candidates = [mk(10, 5), mk(30, 1), mk(30, 9), mk(20, 0)];
		// txg 30 is highest; the timestamp tie-break picks index 2.
		assert_eq!(select_active(&candidates), Some(2));
		assert_eq!(select_active(&[]), None);
	}

	#[test]
	fn mmp_sequence_needs_magic_and_valid_bit() {
		let mut uberblock = Uberblock::parse(&{
			let mut s = blank_slot();
			s[OFF_MAGIC..OFF_MAGIC + 8].copy_from_slice(&UBERBLOCK_MAGIC.to_le_bytes());
			s
		})
		.unwrap();
		uberblock.mmp_magic = MMP_MAGIC;
		uberblock.mmp_config = 0x02 | (7u64 << 32); // seq-valid bit set, sequence = 7
		assert_eq!(uberblock.mmp_sequence(), Some(7));
		uberblock.mmp_config = 7u64 << 32; // valid bit clear
		assert_eq!(uberblock.mmp_sequence(), None);
	}
}
