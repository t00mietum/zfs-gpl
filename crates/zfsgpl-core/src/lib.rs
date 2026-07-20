// SPDX-License-Identifier: GPL-2.0-or-later
//
// Core zfs-gpl logic. Layered SPA -> DMU -> DSL -> ZPL, built on the on-disk
// types. Implemented from spec only; see cleanroom.md. Empty for now.

#![forbid(unsafe_code)]

pub use zfsgpl_ondisk as ondisk;
