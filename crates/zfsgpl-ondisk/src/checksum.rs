// SPDX-License-Identifier: GPL-2.0-or-later
//
// The label/uberblock offset-anchored self-checksum. Spec:
// spec/specs/format/01-vdev-label-uberblock.md sec 6 (construction) and
// spec/specs/format/02-label-checksum-sha256-packing.md (digest-to-word packing).

//! Every fixed label structure - the nvlist area, the boot-env block, and each
//! uberblock slot - ends with a 40-byte trailer: an 8-byte magic then a 256-bit
//! SHA-256 whose input has the block's own device offset injected in place of
//! the stored checksum. That anchoring is what lets a reader validate any slot
//! without first knowing which txg it holds, and stops a stale copy read from
//! the wrong place from validating. (spec 01 sec 6.3)

use crate::sha256::{sha256, Sha256};
use crate::uberblock::ByteOrder;

/// Trailer magic; also encodes writer byte order. (spec 01 sec 6.1, 7)
pub const TRAILER_MAGIC: u64 = 0x0210_da7a_b10c_7a11;

/// Trailer is 8 bytes magic + 32 bytes checksum. (spec 01 sec 6.1)
pub const TRAILER_SIZE: usize = 40;
const CHECKSUM_FIELD_SIZE: usize = 32;

/// Verify a structure's offset-anchored SHA-256 self-checksum.
///
/// `area` is the entire checksummed region (the whole nvlist area, boot-env
/// block, or uberblock slot - including trailing padding); `device_offset` is
/// the absolute byte offset on the device where that region begins. Returns
/// `false` if the area is too small or the trailer magic is unrecognizable, so
/// a caller can use this as the spec 8.1 validity gate directly. (spec 01 sec
/// 6.3, spec 02 sec 4)
#[must_use]
pub fn verify(area: &[u8], device_offset: u64) -> bool {
	let Some(magic_off) = area.len().checked_sub(TRAILER_SIZE) else {
		return false;
	};
	let Some(order) = detect_order(read_u64_le(area, magic_off)) else {
		return false;
	};
	let digest = anchored_digest(area, device_offset, order);
	let expected = digest_words(&digest);
	let field_off = magic_off + 8;
	expected.iter().enumerate().all(|(j, exp)| {
		let raw = read_u64_le(area, field_off + j * 8);
		let stored = match order {
			ByteOrder::Native => raw,
			ByteOrder::Swapped => raw.swap_bytes(),
		};
		stored == *exp
	})
}

/// Write a valid self-checksum trailer into the last 40 bytes of `area`, in
/// little-endian (native) form. The mirror of [`verify`]: set the magic, inject
/// the `{offset,0,0,0}` verifier, hash the whole area, store the packed digest.
/// (spec 01 sec 6.3 "to write")
///
/// # Panics
/// Panics if `area` is shorter than the 40-byte trailer.
pub fn write_trailer(area: &mut [u8], device_offset: u64) {
	build_trailer(area, device_offset, ByteOrder::Native);
}

/// SHA-256 of the area with the `{offset,0,0,0}` verifier standing in for the
/// stored checksum field, hashed in the writer's byte order. Feeds the prefix
/// (through the trailer magic) then the 32-byte verifier, so no copy of the
/// area is needed. (spec 01 sec 6.3 steps 3-5)
fn anchored_digest(area: &[u8], device_offset: u64, order: ByteOrder) -> [u8; 32] {
	let field_off = area.len() - CHECKSUM_FIELD_SIZE;
	let mut hasher = Sha256::new();
	hasher.update(&area[..field_off]);
	hasher.update(&verifier(device_offset, order));
	hasher.finalize()
}

/// The 32-byte verifier: the device offset as one word in writer order, then 24
/// zero bytes (words 1..3 of `{off,0,0,0}`). (spec 01 sec 6.3 step 3)
fn verifier(device_offset: u64, order: ByteOrder) -> [u8; 32] {
	let mut buf = [0u8; 32];
	buf[..8].copy_from_slice(&order_bytes(device_offset, order));
	buf
}

fn build_trailer(area: &mut [u8], device_offset: u64, order: ByteOrder) {
	let magic_off = area.len() - TRAILER_SIZE;
	let field_off = magic_off + 8;
	area[magic_off..field_off].copy_from_slice(&order_bytes(TRAILER_MAGIC, order));
	area[field_off..field_off + CHECKSUM_FIELD_SIZE]
		.copy_from_slice(&verifier(device_offset, order));
	let digest = sha256(area);
	for (j, word) in digest_words(&digest).iter().enumerate() {
		let base = field_off + j * 8;
		area[base..base + 8].copy_from_slice(&order_bytes(*word, order));
	}
}

/// Pack a 32-byte SHA-256 digest into the four trailer words. Each word is the
/// big-endian 64-bit value of eight consecutive digest bytes, so `word[j]`
/// pairs hash words `H(2j)` (high) and `H(2j+1)` (low). (spec 02 sec 2)
fn digest_words(digest: &[u8; 32]) -> [u64; 4] {
	let mut words = [0u64; 4];
	for (word, chunk) in words.iter_mut().zip(digest.chunks_exact(8)) {
		*word = u64::from_be_bytes(chunk.try_into().expect("chunks_exact(8) yields 8 bytes"));
	}
	words
}

/// Recognize the trailer magic read as little-endian, mapping it to the writer's
/// byte order relative to the little-endian decode convention. (spec 01 sec 7)
fn detect_order(magic_le: u64) -> Option<ByteOrder> {
	if magic_le == TRAILER_MAGIC {
		Some(ByteOrder::Native)
	} else if magic_le == TRAILER_MAGIC.swap_bytes() {
		Some(ByteOrder::Swapped)
	} else {
		None
	}
}

/// A `u64` as the eight bytes a writer of `order` would store it as. `Native` is
/// the little-endian decode convention shared with `uberblock`; `Swapped` is a
/// big-endian writer.
fn order_bytes(value: u64, order: ByteOrder) -> [u8; 8] {
	match order {
		ByteOrder::Native => value.to_le_bytes(),
		ByteOrder::Swapped => value.to_be_bytes(),
	}
}

fn read_u64_le(buf: &[u8], offset: usize) -> u64 {
	let mut bytes = [0u8; 8];
	bytes.copy_from_slice(&buf[offset..offset + 8]);
	u64::from_le_bytes(bytes)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::sha256::sha256;

	// spec 02 sec 5: the packing of the two FIPS digests into the four words.
	#[test]
	fn packs_fips_vectors_to_words() {
		assert_eq!(
			digest_words(&sha256(b"")),
			[
				0xe3b0_c442_98fc_1c14,
				0x9afb_f4c8_996f_b924,
				0x27ae_41e4_649b_934c,
				0xa495_991b_7852_b855,
			]
		);
		assert_eq!(
			digest_words(&sha256(b"abc")),
			[
				0xba78_16bf_8f01_cfea,
				0x4141_40de_5dae_2223,
				0xb003_61a3_9617_7a9c,
				0xb410_ff61_f200_15ad,
			]
		);
	}

	#[allow(clippy::cast_possible_truncation)]
	fn area_with(len: usize, fill: u8) -> Vec<u8> {
		let mut area = vec![fill; len];
		// A varied, deterministic body so a corrupted byte is detectable.
		for (i, b) in area.iter_mut().enumerate() {
			*b = fill.wrapping_add(i as u8);
		}
		area
	}

	#[test]
	fn native_round_trip_validates() {
		let mut area = area_with(1024, 0x11);
		write_trailer(&mut area, 0x8000);
		assert!(verify(&area, 0x8000));
	}

	#[test]
	fn wrong_offset_fails() {
		let mut area = area_with(1024, 0x22);
		write_trailer(&mut area, 0x8000);
		// The offset is baked into the hash, so a stale copy read elsewhere fails.
		assert!(!verify(&area, 0x8400));
	}

	#[test]
	fn corrupted_body_fails() {
		let mut area = area_with(2048, 0x33);
		write_trailer(&mut area, 0x40000);
		area[100] ^= 0xff;
		assert!(!verify(&area, 0x40000));
	}

	#[test]
	fn swapped_writer_round_trips() {
		// A big-endian writer's trailer must validate on this little-endian decode.
		let mut area = area_with(1024, 0x44);
		build_trailer(&mut area, 0x8000, ByteOrder::Swapped);
		assert!(verify(&area, 0x8000));
	}

	#[test]
	fn rejects_missing_or_short_trailer() {
		assert!(!verify(&[0u8; 8], 0)); // shorter than the trailer
		assert!(!verify(&[0u8; 1024], 0)); // no valid trailer magic
	}
}
