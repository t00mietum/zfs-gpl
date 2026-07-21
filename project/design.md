<!-- markdownlint-disable MD007 -- Unordered list indentation -->
<!-- markdownlint-disable MD010 -- No hard tabs -->
<!-- markdownlint-disable MD033 -- No inline html -->
<!-- markdownlint-disable MD055 -- Table pipe style [Expected: leading_and_trailing; Actual: leading_only; Missing trailing pipe] -->
<!-- markdownlint-disable MD041 -- First line in a file should be a top-level heading -->
# Design

Design, requirements, and direction. The active pre-v1.0.0 bug/feature task list lives in [`backlog.md`](backlog.md). Process and legal detail live in [`../cleanroom.md`](../cleanroom.md), [`../provenance.md`](../provenance.md), and [`legal/patent-fto.md`](legal/patent-fto.md).

A clean-room, from-scratch reimplementation of ZFS in Rust, licensed GPL-2.0-or-later, that reads and writes the OpenZFS on-disk format.

## Assumptions

Goals:

- On-disk compatible with OpenZFS, both directions.
- Coexists with an installed OpenZFS on the same machine, no conflict.
- Uses the native OS page cache rather than a reimplemented ARC.
- Discovers devices by native enumeration and label scan, not a cache file.
- GPL-licensed, so it is at least a candidate for places OpenZFS cannot go (the mainline Linux tree), and free of the CDDL/GPL bind.

Non-goals (for now):

- Bug-for-bug parity with OpenZFS internals. We match the format and the observable behavior, not the implementation.
- Importing OpenZFS tuning knobs one-for-one.

## Project structure

### Folder structure

Rust workspace at the repo root, member crates under `crates/`. The published on-disk spec is consumed as a submodule at `spec/`.

### Logical code structure

Workspace of crates:

- `zfsgpl-ondisk`: pure on-disk format types and constants. Each item cites its spec source. No I/O.
- `zfsgpl-core`: pool/vdev (SPA), object layer (DMU), dataset layer (DSL), POSIX layer (ZPL).
- `zfsgpl-cli`: the `zgpl` front end.

### Data flow

	device labels / native enumeration
		-> SPA (pool, vdev, uberblock, spacemaps)
			-> DMU (objects, object sets, transactions)
				-> DSL (datasets, snapshots, properties)
					-> ZPL (files, directories, POSIX)

### Execution flow/loops

The `zgpl` CLI drives the stack; the OS page cache sits underneath. Import scans device labels; read follows block pointers down the DMU/DSL/ZPL layers.

## Direction decisions

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

### Licensing and legal posture

- GPL-2.0-or-later, no CDDL code, ever. Independent-creation evidence is maintained continuously (cleanroom.md).
- Patents are a separate risk that clean-room does not address, tracked in `legal/patent-fto.md`, and are for counsel. Going pure-GPL forfeits the CDDL patent grant; that trade-off is accepted deliberately and noted for review.

### Patent-aware build order

Clean-room addresses copyright, not patents. To keep patent exposure low without stalling development, we decided to sequence the roadmap by patent status:

- Features whose relevant patents are expired (or that no patent covers) are built first: the foundational format mechanics - core read/write, pooled storage and dynamic striping, integrity checksums, endianness handling.
- Features that may still be covered are deferred and, until any applicable patent expires, exist as stubs only: enough inert code for the working stack to compile and run.
- A stub may carry the design as comment-only pseudocode - plain English at any level of detail, up to a full step-by-step that later becomes a real implementation over days or weeks. As comments it is unexecutable text, not a machine or a method that runs.
- No patent-covered logic is ever compiled or executed - not in releases, tests, or dev builds. A stub does not practice a claim; pseudocode in a comment is not code.
- A stub is promoted to a real implementation only once its patents have lapsed, or counsel clears it.
- Build order therefore tracks patent expiry, which shapes the phased roadmap. The public-facing form of this commitment is the README "Patents" section; the working patent inventory is kept in private notes, pending counsel's formal freedom-to-operate opinion.

## Plan

Format coverage strategy:

- The durable core (labels, uberblock, block pointers, DMU/DSL/ZAP/ZPL) is specified in the published 2006 document and can be implemented directly from it.
- Everything post-2006 (feature flags, native encryption, dRAID, zstd, log spacemaps, large dnodes and blocks) is documented only in CDDL source, so it comes through the clean-room spec pipeline. This is the bulk of the modern format and the main schedule risk.
- Read support is the near-term target; write support is greenfield everywhere (no independent project has done it) and is the harder, later phase.
- Build order also tracks patent status (see Patent-aware build order): unencumbered format mechanics first; features that may still be covered (self-heal, ditto blocks, resilver, per-dataset redundancy, snapshots/clones, scrub, encryption, secure erase) stay stub-only until their patents lapse.

Phased roadmap and current status are in [`backlog.md`](backlog.md).

## Architecture

### Software stack

Rust. Release profile favors small binary then speed (see the root `Cargo.toml`), per project priorities.

### Rust paradigm and conventions

Read this before writing any Rust in this repo - it is required startup reading (see `CLAUDE.md`). The style is deliberately narrow because a filesystem decodes untrusted, possibly-corrupt bytes and the top priorities are small binary then speed.

Paradigm - data-oriented, boundary-validated, mostly-safe, sync-core:

- Data-oriented, not object-oriented. Model the format as plain structs, enums, and free functions with pattern matching. Use enums for the format's closed sum types (block-pointer and DMU object types, checksum and compression algorithms); reserve traits for real substitution seams. Static dispatch over `dyn` - smaller and faster.
- Parse, don't validate. Decode raw bytes into already-validated, strongly-typed structs at the I/O boundary; inland code only ever sees proven-valid types. Newtype the primitives (`Dva`, `BlockPointer`, `Txg`, `ObjsetId`) so a raw `u64` cannot be cross-wired into the wrong field.
- Errors are values; never panic on disk input. `Result` at every decode boundary, a dedicated error enum, no `unwrap`/`expect`/indexing panics on bytes read from a device. `panic!` is only for violated internal invariants (programmer bugs). Corruption is an expected outcome, not an exception.
- `no_std` + `alloc` in the core, from day one. `zfsgpl-ondisk` and as much of `zfsgpl-core` as possible stay `no_std`; push `std`/OS/syscalls out to the edges (`zfsgpl-cli`, the I/O layer). This keeps a future kernel-module target viable instead of a rewrite.
- Unsafe is quarantined, not sprinkled. `#![forbid(unsafe_code)]` on the pure crates. Do not cast a buffer to a `#[repr(C)]` struct - the format is bi-endian and byte order is detected at runtime, so decode explicitly and endian-parameterized. Where `unsafe` is unavoidable (mmap, ioctls, FFI) isolate it in one small audited module with `// SAFETY:` comments.
- Value semantics fit COW. ZFS is copy-on-write: transactions produce new block versions, not in-place mutation - identical to Rust's move/clone model. Blocks are addressed by on-disk pointers (values), so the block tree is load-on-demand-by-address, never an `Rc<RefCell<_>>` graph.
- Sync core; async only at the I/O edge if ever. Decode is pure CPU. Keep the `Vdev`/`BlockDevice` I/O trait synchronous; do not bake `tokio` into the foundation (a kernel target makes async-Rust awkward).
- Traits at the substitution seam only. The block-device/vdev backend (file, raw device, in-memory tmpfs fixture) is the main one; it is what lets the same core run under FUSE, a kernel shim, or the loopback test harness.

Avoid these LLM-typical weaknesses - write it as an expert would:

- No reflexive `.clone()` to appease the borrow checker. Restructure instead: borrow, split-borrow fields, scope the borrow, use indices, or `mem::take`/`mem::swap`. Clone only when the copy is genuinely needed and its cost is acceptable - never clone a data block or large buffer just to move past a borrow error. A deliberate clone should be obviously justified.
- Never reach for `unsafe` to silence a borrow-checker fight. That is the inverse trap; restructure.
- No needless allocation. Prefer `&str`/`&[u8]`/`Cow` and iterator chains over `to_owned`/`String`/`collect` round-trips. Allocation is binary size and speed, both top priorities here.
- No premature abstraction. No generics, traits, or builders until a real second caller exists. Prefer the simplest concrete thing; generalize when the need is real, not anticipated.
- No `Arc<Mutex<_>>`/interior-mutability reflex for sharing. Reach for ownership and borrows first.
- No swallowed errors (`let _ =`, careless `.ok()`) and no `unwrap`/`expect` on runtime or disk data. Propagate with `?`.
- Prefer std and idiomatic iterators over hand-rolled loops or a pulled-in crate for something std already does - but keep it readable, not clever. Every dependency is binary size and clean-room/supply-chain surface.
- Default to tight visibility (`pub(crate)`), not reflexive `pub`.

### Configuration model

Open design work.

### Saves and persistence

The on-disk format itself is the persistence layer; nothing else is persisted outside the pool.

### UI

`zgpl` CLI drives the stack. No GUI planned.

### Testing

- Independent conformance vectors from the spec.
- Black-box interop harness: round-trip pools between zfs-gpl and stock OpenZFS.
- Derived (not copied) test cases expanding coverage over time.

## Open questions

- Kernel module vs FUSE vs userspace-first for the initial usable milestone.
- How native caching interacts with the write pipeline and with checksums.
- How far back to push write compatibility versus read compatibility.
