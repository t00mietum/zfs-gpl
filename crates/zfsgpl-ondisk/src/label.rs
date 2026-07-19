// SPDX-License-Identifier: GPL-2.0-or-later
//
// VDEV label geometry. Spec: spec/specs/format/01-vdev-label-uberblock.md,
// sections 2 and 4. Pure layout math, no I/O.

//! Where the four labels sit on a leaf device, the regions inside one label,
//! and where the uberblock slots fall. These offsets are the interoperability
//! facts a reader must match to find a pool's entry point.

/// One VDEV label is 256 KiB. (spec 2.1)
pub const LABEL_SIZE: u64 = 256 * 1024;

/// Four labels per leaf device: L0,L1 at the front, L2,L3 at the rear. (spec 2.1)
pub const LABEL_COUNT: u32 = 4;

/// Region 1: blank / VTOC pad, 8 KiB at label offset 0. (spec 2.4)
pub const BLANK_SIZE: u64 = 8 * 1024;

/// Region 2: boot header / boot-env block, 8 KiB. (spec 2.4)
pub const BOOT_HEADER_OFFSET: u64 = 0x2000;
pub const BOOT_HEADER_SIZE: u64 = 8 * 1024;

/// Region 3: config nvlist area, 112 KiB. (spec 2.4)
pub const NVLIST_OFFSET: u64 = 0x4000;
pub const NVLIST_SIZE: u64 = 112 * 1024;

/// Region 4: uberblock ring, 128 KiB. (spec 2.4, 4)
pub const UBERBLOCK_RING_OFFSET: u64 = 0x20000;
pub const UBERBLOCK_RING_SIZE: u64 = 128 * 1024;

/// Each fixed label structure ends with a 40-byte self-checksum trailer. (spec 6.1)
pub const CHECKSUM_TRAILER_SIZE: u64 = 40;

/// Reserved boot region between the front label pair and pool data:
/// 3.5 MiB at device offset 512 KiB. (spec 2.3)
pub const BOOT_REGION_OFFSET: u64 = 0x80000;
pub const BOOT_REGION_SIZE: u64 = 7 << 19;

/// Device byte offset of label `l` (0..3) on a device of `psize` bytes. (spec 2.2)
/// `psize` is treated as a whole multiple of `LABEL_SIZE`; any trailing
/// remainder is unused.
pub fn label_offset(l: u32, psize: u64) -> u64 {
	debug_assert!(l < LABEL_COUNT);
	if l < 2 {
		u64::from(l) * LABEL_SIZE
	} else {
		psize - u64::from(LABEL_COUNT - l) * LABEL_SIZE
	}
}

/// Uberblock slot size = 2^clamp(ashift,10,13), i.e. 1 KiB..8 KiB. (spec 4)
pub fn uberblock_slot_size(ashift: u32) -> u64 {
	1u64 << ashift.clamp(10, 13)
}

/// Number of uberblock slots in one label's ring for a given ashift. (spec 4)
pub fn uberblock_slot_count(ashift: u32) -> u64 {
	UBERBLOCK_RING_SIZE / uberblock_slot_size(ashift)
}

/// Offset within a label of uberblock slot `n`. (spec 4)
pub fn uberblock_slot_offset(n: u64, ashift: u32) -> u64 {
	UBERBLOCK_RING_OFFSET + n * uberblock_slot_size(ashift)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn label_regions_sum_to_label_size() {
		assert_eq!(
			BLANK_SIZE + BOOT_HEADER_SIZE + NVLIST_SIZE + UBERBLOCK_RING_SIZE,
			LABEL_SIZE
		);
		assert_eq!(NVLIST_OFFSET, BLANK_SIZE + BOOT_HEADER_SIZE);
		assert_eq!(UBERBLOCK_RING_OFFSET, NVLIST_OFFSET + NVLIST_SIZE);
	}

	#[test]
	fn front_and_rear_label_positions() {
		let psize = 100 * LABEL_SIZE; // a tidy whole number of labels
		assert_eq!(label_offset(0, psize), 0);
		assert_eq!(label_offset(1, psize), LABEL_SIZE);
		assert_eq!(label_offset(2, psize), psize - 2 * LABEL_SIZE);
		assert_eq!(label_offset(3, psize), psize - LABEL_SIZE);
	}

	#[test]
	fn slot_size_clamps_and_tiles_the_ring() {
		assert_eq!(uberblock_slot_size(9), 1024); // clamped up to 2^10
		assert_eq!(uberblock_slot_size(12), 4096);
		assert_eq!(uberblock_slot_size(15), 8192); // clamped down to 2^13
		assert_eq!(uberblock_slot_count(9), 128);
		assert_eq!(uberblock_slot_count(13), 16);
		assert_eq!(
			uberblock_slot_offset(3, 12),
			UBERBLOCK_RING_OFFSET + 3 * 4096
		);
	}

	#[test]
	fn boot_region_is_three_and_a_half_mib() {
		assert_eq!(BOOT_REGION_SIZE, 3_670_016);
		assert_eq!(BOOT_REGION_OFFSET, 2 * LABEL_SIZE);
	}
}
