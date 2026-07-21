<!-- markdownlint-disable MD007 -- Unordered list indentation -->
<!-- markdownlint-disable MD010 -- No hard tabs -->
<!-- markdownlint-disable MD033 -- No inline html -->
<!-- markdownlint-disable MD055 -- Table pipe style [Expected: leading_and_trailing; Actual: leading_only; Missing trailing pipe] -->
<!-- markdownlint-disable MD041 -- First line in a file should be a top-level heading -->
# Requirements

This is a product backlog just for the pre-v1.0.0 release. After that, bugs, features, and enhancements will be managed in GitHub Issues. Companion: [`design.md`](design.md). Process: [`../cleanroom.md`](../cleanroom.md).

<!-- TOC ignore:true -->
## Table of contents
<!-- TOC -->

- [Conventions](#conventions)
- [Initial requirements](#initial-requirements)
	- [Build, CI/CD, and install](#build-cicd-and-install)
	- [Configuration and persistence](#configuration-and-persistence)
- [Backlog](#backlog)
	- [Phase 0 - foundation](#phase-0---foundation)
	- [Phase 1 - legal groundwork](#phase-1---legal-groundwork)
	- [Phase 2 - read the core format](#phase-2---read-the-core-format)
	- [Phase 3 - modern format features](#phase-3---modern-format-features)
	- [Phase 4 - write path](#phase-4---write-path)
	- [Phase 5 - system integration](#phase-5---system-integration)
	- [Testing (ongoing)](#testing-ongoing)
	- [Future and/or deferred](#future-andor-deferred)
	- [Canceled](#canceled)

<!-- /TOC -->

## Conventions

In each section, items are listed approximately from newest to oldest.

| Icon | Status
| :--: | :--
| 🔘   | Not started
| 🛠️   | Started, and/or partially complete
| ✋   | Defer
| ✅   | Complete
| 🚫   | Canceled

## Initial requirements

### Build, CI/CD, and install

- 🔘 A CI/CD pipeline kicked off by a bash script (`cicd/cicd.bash`): builds, tests, and can commit and push. Packaging and publishing are opt-in.
- 🔘 Dev-environment install script (Linux bash, macOS sh, Windows PowerShell), runnable via a single `curl`/`wget`. Clones main, installs dependencies, and states what it will do with an option to abort.
- 🔘 Release-install script per platform, runnable via a single `curl`/`wget`. Downloads, installs, and runs the latest release, with an option to abort.

### Configuration and persistence

- 🔘 Default configuration hard-coded
	- 🔘 Overridden by per-user config file, created the first time a default setting is changed.
		- 🔘 Settings live under `~/.config` (YAML or TOML), resistant to errors (e.g. don't bail on the whole thing due to one bad line).
	- 🔘 Overridden by program options at run-time.

## Backlog

### Phase 0 - foundation

- 🛠️ Repo, license, provenance, clean-room process, workspace skeleton
	- ✅ GPL-2.0-or-later license, notice, provenance, contributing
	- ✅ clean-room process document
	- ✅ Rust workspace (`zfsgpl-ondisk`, `zfsgpl-core`, `zfsgpl-cli`)
	- ✅ Design and backlog docs
	- ✅ Set up the spec repo (`zfs-gpl-spec`) and wire the dirty tree with its own role-constraining context (`DIRTY-SIDE.md`, openzfs pinned)
	- 🔘 CI: build, test, fmt, clippy (the clone-detection tripwire runs spec-side, not here - see Phase 1)
	- 🔘 rustfmt/clippy pre-commit hook

### Phase 1 - legal groundwork (parallel, some for counsel)

- 🛠️ Physical wall: isolated spec environment on a separate system (no network/filesystem path to this one), with its own agent installation, instructions, memory, and settings
	- Dirty tree and spec working tree move there; no OpenZFS source copy remains on the implementation system
- ✅ Clean-room process v2: physical wall, single crossing point + request channel, similarity audit, evidence retention, shared-model section (`cleanroom.md`)
- ✅ Request channel: `requests.md` in the spec repo - questions in, approved specs out, all logged in git
- ✅ Evidence retention: transcript retention raised, daily append-only archive with SHA-256 manifest under `private/` (outside this repo)
- 🔘 Similarity audit script (spec-side): lexical harvest (identifiers/strings/comments/constants) of this repo vs pinned source, allowlist from approved specs, results to the gatekeeper log
- 🛠️ Freedom-to-operate patent search on Sun/Oracle ZFS patents and third-party filesystem patents (counsel-led); `legal/patent-fto.md`. Preliminary bibliographic inventory done 2026-07-20 (live vs expired, real dates) but kept in PRIVATE notes, not public (willfulness caution); counsel-led claim reads + two gaps (RAID-Z parity, foundational COW grant) still open
- 🔘 Counsel review of cleanroom.md (incl. the shared-model section) and the CDDL-patent-grant trade-off
- 🔘 Confirm the published 2006 spec is the format-of-record for the pre-2006 core, and catalogue what it does and does not cover
- ✅ Stand up the spec pipeline: dirty-side analysis -> gatekeeper review -> spec repo, with the evidence trail (first spec: label+uberblock, `7f86d1b`)

### Phase 2 - read the core format

- 🛠️ Vdev labels: geometry/region math done (`label.rs`), self-checksum validation done (`checksum.rs`); nvlist (XDR) parse still open
- 🛠️ Uberblock: parse + endianness + active-uberblock ranking done (`uberblock.rs`); device I/O layer now scans real slots
	- ✅ Device I/O seam: `BlockDevice` trait + `FileDevice` (image or raw device), portable positioned reads (`device.rs`)
	- ✅ Leaf-vdev scan: read all readable labels, discover uberblock candidates at 1 KiB stride (ashift-independent), rank to the active one (`vdev.rs`); wired to `zgpl scan <path>`
	- ✅ Self-checksum gate (spec 8.1 step 3): each candidate must self-checksum against its device offset; tries all slot sizes so it stays ashift-independent
- ✅ Label/uberblock self-checksum: offset-anchored SHA-256 (spec 6), digest-to-word packing per spec 02. Own SHA-256 (`sha256.rs`, FIPS 180-4), no crypto dep; verified against FIPS + spec-02 packing vectors
- 🔘 Block pointers and DVAs: decode, follow, verify checksums (fletcher4, SHA-256)
- 🔘 DMU: dnodes and object sets
- 🔘 ZAP: micro and fat
- 🔘 DSL: enumerate datasets and snapshots
- 🔘 ZPL: read files and directories
- 🔘 `zgpl` read-only: import a pool and list/read a dataset

### Phase 3 - modern format features (via clean-room spec pipeline)

- 🔘 Feature flags: detect and gate
- 🔘 Compression: lz4, zstd, gzip
- 🔘 Checksums beyond the core: edonr, blake3, skein
- 🔘 Native encryption (note: illumos vs OpenZFS formats differ)
- 🔘 dRAID and raidz geometry
- 🔘 Log spacemaps, large dnodes, large/embedded blocks

### Phase 4 - write path (greenfield)

- 🔘 Allocation and spacemaps
- 🔘 Transaction groups and the copy-on-write commit
- 🔘 ZIL
- 🔘 Create/modify/snapshot, round-trip verified against OpenZFS as external oracle

### Phase 5 - system integration

- 🔘 Native OS caching design and implementation
- 🔘 Native device discovery (no cache-file dependence)
- 🔘 Delivery form: kernel module vs FUSE vs userspace, decided and built
- 🔘 Coexistence validation with a live OpenZFS install

### Testing (ongoing)

- 🔘 Independent conformance vectors from the spec
- 🔘 Black-box interop harness: round-trip pools between zfs-gpl and stock OpenZFS
- 🔘 Derived (not copied) test cases expanding coverage over time

### Future and/or deferred

### Canceled
