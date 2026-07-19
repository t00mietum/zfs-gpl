// SPDX-License-Identifier: GPL-2.0-or-later
//
// On-disk format types for zfs-gpl.
//
// Every constant and layout here carries a provenance note pointing at its
// source in the published ZFS on-disk format spec (Sun, 2006) or, for
// post-2006 features, at the gatekeeper-approved spec-repo document it came
// from. No value in this crate is taken from OpenZFS source. See CLEANROOM.md.

#![forbid(unsafe_code)]

/// Uberblock magic, `0x00bab10c` ("oo-ba-block").
/// Provenance: published on-disk format spec, uberblock section.
pub const UBERBLOCK_MAGIC: u64 = 0x0000_0000_00ba_b10c;

/// Physical vdev label size, 256 KiB.
/// Provenance: published on-disk format spec, vdev label section.
pub const VDEV_LABEL_SIZE: u64 = 256 * 1024;

/// Number of vdev labels per device (L0,L1 at the front, L2,L3 at the end).
/// Provenance: published on-disk format spec, vdev label section.
pub const VDEV_LABEL_COUNT: u32 = 4;

#[cfg(test)]
mod tests {
	#[test]
	fn label_geometry_is_sane() {
		assert_eq!(super::VDEV_LABEL_COUNT, 4);
		assert_eq!(super::VDEV_LABEL_SIZE, 262_144);
	}
}
