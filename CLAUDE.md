<!-- markdownlint-disable MD007 -- Unordered list indentation -->
<!-- markdownlint-disable MD010 -- No hard tabs -->
<!-- markdownlint-disable MD033 -- No inline html -->
<!-- markdownlint-disable MD055 -- Table pipe style [Expected: leading_and_trailing; Actual: leading_only; Missing trailing pipe] -->
<!-- markdownlint-disable MD041 -- First line in a file should be a top-level heading -->
# CLAUDE.md

Living project memory (state, dev/test workflow): @.claude/memory.md

## Clean-room (read first)

This is a clean-room GPLv2+ reimplementation of ZFS. This context is the IMPLEMENTATION side: never read OpenZFS source (no cat/Read/grep-with-content). The published 2006 on-disk spec is safe (it's a document). Post-2006 features come only via the spec pipeline. Full rules: `github/cleanroom.md`. Why it matters and current state: `.claude/memory.md`.

## Attribution (this project)

- `Instance:` here is always `zfs-gpl-implementation`. The spec environment uses `zfs-gpl-spec`.

- The Instance trailer doubles as clean-room evidence: it records which isolated side authored each commit.

- The remote is the SSH alias `git@github_t00mietum:...`, not literal `github.com`, and it is transport only. Commits before 2026-07-19 were authored as t00mietum. `gh` is authed under my personal account; keep that account out of history and content.

## Build / run

- Rust workspace under `github/`. `cd github && cargo build --release`; `cargo test`; `cargo fmt --all` (rustfmt pinned `hard_tabs = true`).

- Binary is `zgpl` (never `zpool`/`zfs` - must coexist with OpenZFS).

## Architecture

- `crates/zfsgpl-ondisk`: on-disk format types/constants, each citing its spec source, no I/O.

- `crates/zfsgpl-core`: SPA (pool/vdev) -> DMU (objects) -> DSL (datasets) -> ZPL (POSIX).

- `crates/zfsgpl-cli`: `zgpl` front end.

- Design: `github/project/design.md`. Roadmap: `github/project/backlog.md`.

- Rust conventions: REQUIRED reading every startup/`/clear`, before writing any Rust - `project/design.md` -> "Rust paradigm and conventions" (data-oriented, parse-don't-validate, `no_std` core, unsafe quarantined; and the anti-LLM-tendency rules: no reflexive `.clone()`, no needless alloc, no premature abstraction).

- Prose and general code/comment/commit style: `style-guide.md` (canonical, human-facing); it points to design.md for the deep Rust paradigm.

## Non-obvious gotchas (all hit during bring-up - don't regress these)

- rustfmt `hard_tabs = true` keeps tab indentation rustfmt-canonical; don't remove it.
