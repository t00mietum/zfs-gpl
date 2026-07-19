# zfs-gpl backlog

Status keys: 🔘 not started · 🛠️ in progress · ✅ done · ✋ deferred · 🚫 cancelled.

Companion: [`design.md`](design.md). Process: [`../CLEANROOM.md`](../CLEANROOM.md).

## Phase 0 - foundation

- 🛠️ Repo, license, provenance, clean-room process, workspace skeleton
	- ✅ GPL-2.0-or-later license, NOTICE, PROVENANCE, CONTRIBUTING
	- ✅ CLEANROOM process document
	- ✅ Rust workspace (`zfsgpl-ondisk`, `zfsgpl-core`, `zfsgpl-cli`)
	- ✅ Design and backlog docs
	- ✅ Set up the spec repo (`zfs-gpl-spec`) and wire the dirty tree with its own role-constraining context (`DIRTY-SIDE.md`, openzfs pinned)
	- 🔘 CI: build, test, fmt, clippy; add the clone-detection tripwire against the OpenZFS corpus
	- 🔘 rustfmt/clippy pre-commit hook

## Phase 1 - legal groundwork (parallel, some for counsel)

- 🔘 Freedom-to-operate patent search on Sun/Oracle ZFS patents and third-party filesystem patents (counsel-led); record in `legal/patent-fto.md`
- 🔘 Counsel review of CLEANROOM.md and the CDDL-patent-grant trade-off
- 🔘 Confirm the published 2006 spec is the format-of-record for the pre-2006 core, and catalogue what it does and does not cover
- ✅ Stand up the spec pipeline: dirty-side analysis -> gatekeeper review -> spec repo, with the evidence trail (first spec: label+uberblock, `7f86d1b`)

## Phase 2 - read the core format

- 🛠️ Vdev labels: geometry/region math done (`label.rs`); nvlist (XDR) parse and checksum validation still open
- 🛠️ Uberblock: parse + endianness + active-uberblock ranking done (`uberblock.rs`); needs the device I/O layer to scan real slots
- 🔘 Label/uberblock self-checksum: offset-anchored SHA-256 (spec 6). Note: digest-to-word packing needs an interop fixture before claiming OpenZFS compat
- 🔘 Block pointers and DVAs: decode, follow, verify checksums (fletcher4, SHA-256)
- 🔘 DMU: dnodes and object sets
- 🔘 ZAP: micro and fat
- 🔘 DSL: enumerate datasets and snapshots
- 🔘 ZPL: read files and directories
- 🔘 `zgpl` read-only: import a pool and list/read a dataset

## Phase 3 - modern format features (via clean-room spec pipeline)

- 🔘 Feature flags: detect and gate
- 🔘 Compression: lz4, zstd, gzip
- 🔘 Checksums beyond the core: edonr, blake3, skein
- 🔘 Native encryption (note: illumos vs OpenZFS formats differ)
- 🔘 dRAID and raidz geometry
- 🔘 Log spacemaps, large dnodes, large/embedded blocks

## Phase 4 - write path (greenfield)

- 🔘 Allocation and spacemaps
- 🔘 Transaction groups and the copy-on-write commit
- 🔘 ZIL
- 🔘 Create/modify/snapshot, round-trip verified against OpenZFS as external oracle

## Phase 5 - system integration

- 🔘 Native OS caching design and implementation
- 🔘 Native device discovery (no cache-file dependence)
- 🔘 Delivery form: kernel module vs FUSE vs userspace, decided and built
- 🔘 Coexistence validation with a live OpenZFS install

## Testing (ongoing)

- 🔘 Independent conformance vectors from the spec
- 🔘 Black-box interop harness: round-trip pools between zfs-gpl and stock OpenZFS
- 🔘 Derived (not copied) test cases expanding coverage over time
