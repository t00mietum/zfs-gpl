// SPDX-License-Identifier: GPL-2.0-or-later
//
// Scan a leaf vdev's four labels and their uberblock rings, off a real device
// or image. Spec: spec/specs/format/01-vdev-label-uberblock.md, sections 2, 4, 8.

//! Finding a pool's entry point on one leaf device: read each label's uberblock
//! ring, parse every slot that holds an uberblock, and rank the candidates to
//! pick the active one (highest txg, then the section-8.3 tie-breaks).
//!
//! The ring is scanned at 1 KiB granularity - the 2006 baseline slot size and
//! the minimum a modern `ashift` can select (spec 4). Every real slot begins on
//! a 1 KiB boundary for every valid ashift, so this discovers every uberblock
//! without first parsing the config nvlist to learn the exact slot size. Padding
//! and larger-slot interiors simply fail the magic check.
//!
//! Each magic hit then passes the spec 8.1 step-3 self-checksum gate: the
//! offset-anchored SHA-256 over the whole slot (spec 6.3). The slot size is
//! `2^clamp(ashift,10,13)`, still unknown at scan time, so the gate tries each
//! possible size - only the true one validates, keeping the scan ashift-free.

use zfsgpl_ondisk::checksum;
use zfsgpl_ondisk::label;
use zfsgpl_ondisk::uberblock::{selection_rank, Uberblock, UBERBLOCK_FIXED_SIZE};

use crate::device::{BlockDevice, DeviceError};

/// Scan stride: the minimum uberblock slot size, 1 KiB (spec 4, 2006 baseline).
const SCAN_STRIDE: usize = 1024;

/// The possible uberblock slot sizes, `2^clamp(ashift,10,13)` (spec 4). The
/// self-checksum gate tries each because the true `ashift` is not yet known.
const SLOT_SIZES: [usize; 4] = [1024, 2048, 4096, 8192];

/// Uberblock ring length as a `usize` for buffer/index math, kept in lockstep
/// with the u64 spec constant used for device-offset arithmetic.
const RING_LEN: usize = 128 * 1024;
const _: () = assert!(RING_LEN as u64 == label::UBERBLOCK_RING_SIZE);

/// One uberblock found on the device, with where it came from. `device_offset`
/// is the absolute byte offset of the slot - the anchor its self-checksum was
/// verified against (spec 6.3).
#[derive(Clone, Debug)]
pub struct UberblockCandidate {
	pub label_idx: u32,
	pub slot_index: u64,
	pub device_offset: u64,
	pub uberblock: Uberblock,
}

/// The result of scanning one leaf device: every uberblock candidate found, and
/// the index of the active one within `candidates` (highest-ranked, spec 8.2).
#[derive(Clone, Debug, Default)]
pub struct VdevScan {
	pub candidates: Vec<UberblockCandidate>,
	pub active: Option<usize>,
}

impl VdevScan {
	/// The active uberblock candidate, if any survived the scan.
	#[must_use]
	pub fn active(&self) -> Option<&UberblockCandidate> {
		self.active.and_then(|i| self.candidates.get(i))
	}
}

/// Device byte offsets of the labels actually readable on a device of `size`
/// bytes: the front pair always, the rear pair when the device is large enough
/// that they neither run past the end nor collide with the front pair. Returned
/// smallest-first with duplicates removed. (spec 2.2)
fn readable_label_offsets(size: u64) -> Vec<u64> {
	let mut offsets: Vec<u64> = (0..label::LABEL_COUNT)
		.map(|idx| label::label_offset(idx, size))
		.filter(|&off| off + label::LABEL_SIZE <= size)
		.collect();
	offsets.sort_unstable();
	offsets.dedup();
	offsets
}

/// Scan every readable label's uberblock ring on `device` and rank the results.
/// A label whose ring cannot be read is skipped (spec 8.4: partial damage must
/// not defeat the scan); a read error is only surfaced when it is not confined
/// to a single label's ring.
///
/// # Errors
/// Returns [`DeviceError`] only for a failure that is not a per-label ring read
/// - currently none beyond propagating an unexpected device fault.
pub fn scan<D: BlockDevice>(device: &D) -> Result<VdevScan, DeviceError> {
	let size = device.size();
	let mut candidates = Vec::new();
	let mut ring = vec![0u8; RING_LEN];

	for (label_idx, label_off) in (0u32..).zip(readable_label_offsets(size)) {
		let ring_off = label_off + label::UBERBLOCK_RING_OFFSET;
		// A ring we cannot read is a damaged/absent label copy: skip it (spec 8.4).
		if device.read_at(ring_off, &mut ring).is_err() {
			continue;
		}
		for (slot_index, chunk) in ring.chunks_exact(SCAN_STRIDE).enumerate() {
			let Some(slot) = chunk.get(..UBERBLOCK_FIXED_SIZE) else {
				continue;
			};
			let Some(uberblock) = Uberblock::parse(slot) else {
				continue;
			};
			let within = slot_index * SCAN_STRIDE;
			let device_offset = ring_off + within as u64;
			// spec 8.1 step 3: the slot must self-checksum against its offset.
			if !checksum_passes(&ring, within, device_offset) {
				continue;
			}
			candidates.push(UberblockCandidate {
				label_idx,
				slot_index: slot_index as u64,
				device_offset,
				uberblock,
			});
		}
	}

	let active = candidates
		.iter()
		.enumerate()
		.max_by_key(|(_, candidate)| selection_rank(&candidate.uberblock))
		.map(|(i, _)| i);

	Ok(VdevScan { candidates, active })
}

/// The self-checksum validity gate (spec 8.1 step 3). The uberblock checksum
/// covers its whole slot, sized `2^clamp(ashift,10,13)` - unknown here - so try
/// each candidate size. The offset-anchored SHA-256 validates for only the true
/// size (a wrong size hashes the wrong byte range), so trying all four keeps the
/// scan ashift-independent.
fn checksum_passes(ring: &[u8], within: usize, device_offset: u64) -> bool {
	SLOT_SIZES.iter().any(|&slot_size| {
		ring.get(within..within + slot_size)
			.is_some_and(|area| checksum::verify(area, device_offset))
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_support::MemDevice;
	use zfsgpl_ondisk::uberblock::UBERBLOCK_MAGIC;

	// Field offsets within a slot, mirrored from the spec (section 5) so tests
	// can plant uberblocks without depending on ondisk-private constants.
	const OFF_MAGIC: usize = 0;
	const OFF_TXG: usize = 16;
	const OFF_TIMESTAMP: usize = 32;

	/// A device of `labels` whole labels, all bytes zero (no uberblocks yet).
	fn blank_device(labels: u64) -> MemDevice {
		let len = usize::try_from(labels * label::LABEL_SIZE).unwrap();
		MemDevice {
			bytes: vec![0u8; len],
		}
	}

	/// Plant a native-order uberblock at (label offset, slot) with a given txg,
	/// then seal it with a valid 1 KiB-slot self-checksum trailer anchored to its
	/// device offset, so it clears the scan's validity gate.
	fn plant(dev: &mut MemDevice, label_off: u64, slot_index: u64, txg: u64, timestamp: u64) {
		let base = usize::try_from(
			label_off + label::UBERBLOCK_RING_OFFSET + slot_index * SCAN_STRIDE as u64,
		)
		.unwrap();
		dev.bytes[base + OFF_MAGIC..base + OFF_MAGIC + 8]
			.copy_from_slice(&UBERBLOCK_MAGIC.to_le_bytes());
		dev.bytes[base + OFF_TXG..base + OFF_TXG + 8].copy_from_slice(&txg.to_le_bytes());
		dev.bytes[base + OFF_TIMESTAMP..base + OFF_TIMESTAMP + 8]
			.copy_from_slice(&timestamp.to_le_bytes());
		checksum::write_trailer(&mut dev.bytes[base..base + SCAN_STRIDE], base as u64);
	}

	#[test]
	fn front_and_rear_labels_are_scanned() {
		let size = 8 * label::LABEL_SIZE;
		assert_eq!(
			readable_label_offsets(size),
			vec![
				0,
				label::LABEL_SIZE,
				size - 2 * label::LABEL_SIZE,
				size - label::LABEL_SIZE,
			]
		);
	}

	#[test]
	fn tiny_device_dedups_overlapping_labels() {
		// Exactly two labels: front L0,L1 and rear L2,L3 land on the same two
		// offsets, so only two distinct labels are scanned.
		assert_eq!(
			readable_label_offsets(2 * label::LABEL_SIZE),
			vec![0, label::LABEL_SIZE]
		);
	}

	#[test]
	fn finds_planted_uberblocks_and_picks_highest_txg() {
		let mut dev = blank_device(8);
		let rear = 8 * label::LABEL_SIZE - label::LABEL_SIZE; // L3
		plant(&mut dev, 0, 0, 10, 100); // L0, slot 0
		plant(&mut dev, 0, 5, 40, 100); // L0, slot 5 - highest txg
		plant(&mut dev, rear, 3, 30, 100); // L3, slot 3
		let scan = scan(&dev).unwrap();
		assert_eq!(scan.candidates.len(), 3);
		let active = scan.active().unwrap();
		assert_eq!(active.uberblock.txg, 40);
		assert_eq!(active.slot_index, 5);
		assert_eq!(
			active.device_offset,
			label::UBERBLOCK_RING_OFFSET + 5 * SCAN_STRIDE as u64
		);
	}

	#[test]
	fn timestamp_breaks_a_txg_tie() {
		let mut dev = blank_device(8);
		plant(&mut dev, 0, 0, 30, 100);
		plant(&mut dev, 0, 1, 30, 900); // same txg, newer timestamp
		let scan = scan(&dev).unwrap();
		assert_eq!(scan.active().unwrap().uberblock.timestamp, 900);
	}

	#[test]
	fn blank_device_yields_no_candidates() {
		let scan = scan(&blank_device(8)).unwrap();
		assert!(scan.candidates.is_empty());
		assert!(scan.active().is_none());
	}

	#[test]
	fn magic_without_valid_checksum_is_rejected() {
		// A slot with correct magic but no self-checksum trailer must not pass
		// the spec 8.1 gate, even though it parses structurally.
		let mut dev = blank_device(8);
		let base = usize::try_from(label::UBERBLOCK_RING_OFFSET).unwrap();
		dev.bytes[base + OFF_MAGIC..base + OFF_MAGIC + 8]
			.copy_from_slice(&UBERBLOCK_MAGIC.to_le_bytes());
		dev.bytes[base + OFF_TXG..base + OFF_TXG + 8].copy_from_slice(&99u64.to_le_bytes());
		let scan = scan(&dev).unwrap();
		assert!(scan.candidates.is_empty());
	}

	#[test]
	fn finds_a_larger_slot_uberblock() {
		// An ashift=12 pool uses 4 KiB slots; the checksum covers the whole slot.
		// The gate must discover it by trying the larger size, not just 1 KiB.
		let mut dev = blank_device(8);
		let base = usize::try_from(label::UBERBLOCK_RING_OFFSET).unwrap();
		let slot_size = 4096usize;
		dev.bytes[base + OFF_MAGIC..base + OFF_MAGIC + 8]
			.copy_from_slice(&UBERBLOCK_MAGIC.to_le_bytes());
		dev.bytes[base + OFF_TXG..base + OFF_TXG + 8].copy_from_slice(&7u64.to_le_bytes());
		checksum::write_trailer(&mut dev.bytes[base..base + slot_size], base as u64);
		let scan = scan(&dev).unwrap();
		assert_eq!(scan.candidates.len(), 1);
		assert_eq!(scan.active().unwrap().uberblock.txg, 7);
	}
}
