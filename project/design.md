# zfs-gpl design

High-level design and the decisions behind it. Process and legal detail live in [`../CLEANROOM.md`](../CLEANROOM.md), [`../PROVENANCE.md`](../PROVENANCE.md), and [`legal/patent-fto.md`](legal/patent-fto.md). Roadmap is in [`backlog.md`](backlog.md).

## What this is

A clean-room, from-scratch reimplementation of ZFS in Rust, licensed GPL-2.0-or-later, that reads and writes the OpenZFS on-disk format.

## Goals

- On-disk compatible with OpenZFS, both directions.
- Coexists with an installed OpenZFS on the same machine, no conflict.
- Uses the native OS page cache rather than a reimplemented ARC.
- Discovers devices by native enumeration and label scan, not a cache file.
- GPL-licensed, so it is at least a candidate for places OpenZFS cannot go (the mainline Linux tree), and free of the CDDL/GPL bind.

## Non-goals (for now)

- Bug-for-bug parity with OpenZFS internals. We match the format and the observable behavior, not the implementation.
- Importing OpenZFS tuning knobs one-for-one.

## Decisions

### Compatibility target

- On-disk compatibility is maintained back to one prior stable minor release of OpenZFS, or three months, whichever is longer. Older formats are read best-effort.
- Version naming follows OpenZFS's scheme, so the correspondence is legible.

### Coexistence with OpenZFS

Both must live on one system without stepping on each other. So:

- Binaries are named distinctly (`zgpl`, not `zpool`/`zfs`).
- Any kernel component uses a distinct module name and device node.
- No shared state-file paths. In particular we do not read or write OpenZFS's cache file.
- Pool import is by scanning device labels, so a pool created by either implementation is discoverable by ours without shared state.

### Native caching

- Lean on the OS page cache instead of carrying ZFS's own ARC. This is one of the main departures from OpenZFS and a reason for the rewrite.
- Consequences (write path, checksum-on-read, memory pressure interaction) are open design work, tracked in the backlog.

### Device discovery

- One of the named gripes with OpenZFS is over-reliance on a cache file for discovery. We enumerate devices natively and scan for valid labels, treating the cache file as an optional accelerator at most, never the source of truth.

### Language and shape

- Rust. Workspace of crates:
	- `zfsgpl-ondisk`: pure on-disk format types and constants. Each item cites its spec source. No I/O.
	- `zfsgpl-core`: pool/vdev (SPA), object layer (DMU), dataset layer (DSL), POSIX layer (ZPL).
	- `zfsgpl-cli`: the `zgpl` front end.
- Release profile favors small binary then speed (see the root `Cargo.toml`), per project priorities.

### Format coverage strategy

- The durable core (labels, uberblock, block pointers, DMU/DSL/ZAP/ZPL) is specified in the published 2006 document and can be implemented directly from it.
- Everything post-2006 (feature flags, native encryption, dRAID, zstd, log spacemaps, large dnodes and blocks) is documented only in CDDL source, so it comes through the clean-room spec pipeline. This is the bulk of the modern format and the main schedule risk.
- Read support is the near-term target; write support is greenfield everywhere (no independent project has done it) and is the harder, later phase.

### Licensing and legal posture

- GPL-2.0-or-later, no CDDL code, ever. Independent-creation evidence is maintained continuously (CLEANROOM.md).
- Patents are a separate risk that clean-room does not address, tracked in `legal/patent-fto.md`, and are for counsel. Going pure-GPL forfeits the CDDL patent grant; that trade-off is accepted deliberately and noted for review.

## Architecture sketch

	device labels / native enumeration
		-> SPA (pool, vdev, uberblock, spacemaps)
			-> DMU (objects, object sets, transactions)
				-> DSL (datasets, snapshots, properties)
					-> ZPL (files, directories, POSIX)
	zgpl CLI drives the stack; OS page cache underneath.

## Open questions

- Kernel module vs FUSE vs userspace-first for the initial usable milestone.
- How native caching interacts with the write pipeline and with checksums.
- How far back to push write compatibility versus read compatibility.
