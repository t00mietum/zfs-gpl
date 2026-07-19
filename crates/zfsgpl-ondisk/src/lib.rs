// SPDX-License-Identifier: GPL-2.0-or-later
//
// On-disk format types for zfs-gpl. Implemented from the approved, fact-only
// spec pinned under `spec/specs/format/`, never from OpenZFS source. Each item
// cites the spec section it derives from. See CLEANROOM.md.

#![forbid(unsafe_code)]

pub mod label;
pub mod uberblock;

pub use uberblock::{ByteOrder, Uberblock, UBERBLOCK_MAGIC};
