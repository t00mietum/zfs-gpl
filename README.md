# zfs-gpl

An independent, clean-room reimplementation of the ZFS filesystem in Rust, licensed **GPL-2.0-or-later**.

The goal is a from-scratch ZFS that reads and writes the same on-disk format as OpenZFS, so the two can coexist on one machine, while shedding some of OpenZFS's design constraints (its own ARC in place of native OS caching, reliance on a cache file for device discovery, and the CDDL licensing that keeps it out of the mainline kernel).

## Status

Pre-alpha. Nothing works yet. This tree currently holds the workspace skeleton, the design, and the clean-room process that governs how the code may be written.

## Why "GPL"

OpenZFS is CDDL, which is incompatible with GPLv2 and is the reason ZFS has never shipped in the Linux kernel tree. This project shares **no** code with OpenZFS. It is an independent work built from the published on-disk format and behavioral observation, and is licensed GPL-2.0-or-later. See [`PROVENANCE.md`](PROVENANCE.md).

## Clean-room

Because the point of the relicense is that the code is genuinely not derived from CDDL source, the project runs a strict two-role separation: one side may study OpenZFS and writes only functional specifications; the other side implements from those specifications and never sees the source. The rules, and why they exist, are in [`CLEANROOM.md`](CLEANROOM.md). Contributors must read [`CONTRIBUTING.md`](CONTRIBUTING.md) first.

## Design

- On-disk compatible with OpenZFS, targeting one prior stable minor release or three months, whichever is longer.
- Version naming follows OpenZFS.
- Coexists with an installed OpenZFS: distinct binary and module names, no shared device paths or state files.
- Native OS page cache instead of a reimplemented ARC.
- Device discovery by native enumeration and label scan, not a cache file.

Full design in [`project/design.md`](project/design.md); roadmap in [`project/backlog.md`](project/backlog.md).

## License

GPL-2.0-or-later. See [`LICENSE`](LICENSE).

This is not legal advice; the clean-room and patent notes in this repo are an engineering process, and are pending review by counsel.
