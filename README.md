<!-- markdownlint-disable MD007 -- Unordered list indentation -->
<!-- markdownlint-disable MD010 -- No hard tabs -->
<!-- markdownlint-disable MD033 -- No inline html -->
<!-- markdownlint-disable MD055 -- Table pipe style [Expected: leading_and_trailing; Actual: leading_only; Missing trailing pipe] -->
<!-- markdownlint-disable MD041 -- First line in a file should be a top-level heading -->
<div align="center">

[![License: GPL v2+](https://img.shields.io/badge/License-GPLv2%2B-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.html)
![Lifecycle: Pre-alpha](https://img.shields.io/badge/Lifecycle-Pre--alpha-red)
![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)

</div>
<!--
[![!#/bin/bash](https://img.shields.io/badge/-%23!%2Fbin%2Fbash-1f425f.svg?logo=gnu-bash)](https://www.gnu.org/software/bash/)
[![made-with-python](https://img.shields.io/badge/Made%20with-Python-1f425f.svg)](https://www.python.org/)
[![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)
![Go](https://img.shields.io/badge/Go-00ADD8?logo=go&logoColor=white)
![Made with](https://img.shields.io/badge/Made%20with-C%2B%2B-brightgreen?style=plastic)
![Made with](https://img.shields.io/badge/Made%20with-Unreal%20Engine-critical?style=plastic)
[![made-with-javascript](https://img.shields.io/badge/Made%20with-JavaScript-1f425f.svg)](https://www.javascript.com)
![License: GPL v2](https://img.shields.io/badge/License-GPLv2-blue.svg)
[![License: GPL v2+](https://img.shields.io/badge/License-GPLv2%2B-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.html)
![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://opensource.org/licenses/MPL-2.0)
![Lifecycle: Alpha](https://img.shields.io/badge/Lifecycle-Alpha-orange)
![Lifecycle: Beta](https://img.shields.io/badge/Lifecycle-Beta-yellow)
![Lifecycle: RC](https://img.shields.io/badge/Lifecycle-RC-blue)
![Lifecycle: Stable](https://img.shields.io/badge/Lifecycle-Stable-brightgreen)
![Lifecycle: Deprecated](https://img.shields.io/badge/Lifecycle-Deprecated-red)
![Status: Deprecated](https://img.shields.io/badge/Status-Deprecated-orange)
![Status: Archived](https://img.shields.io/badge/Status-Archived-lightgrey)
![Lifecycle: EOL](https://img.shields.io/badge/Lifecycle-EOL-lightgrey)
![Coverage](https://img.shields.io/badge/Coverage-25%25-red)
![Coverage](https://img.shields.io/badge/Coverage-50%25-orange)
![Coverage](https://img.shields.io/badge/Coverage-75%25-yellow)
![Coverage](https://img.shields.io/badge/Coverage-90%25-brightgreen)
![Status: Passing](https://img.shields.io/badge/Status-Passing-brightgreen)
![Status: Failing](https://img.shields.io/badge/Status-Failing-red)
[![GitHub Sponsors](https://img.shields.io/github/sponsors/t00mietum?logo=GitHub%20Sponsors&style=social)](https://github.com/sponsors/t00mietum)
-->

<!-- TOC ignore:true -->
# ZFS-GPL

An independent, clean-room reimplementation of the ZFS filesystem in Rust, licensed **GPL-2.0-or-later**. It reads and writes the same on-disk format as OpenZFS, so the two can coexist on one machine, while shedding some of OpenZFS's design constraints.

<!-- TOC ignore:true -->
## Table of contents

<!-- TOC -->

- [Why](#why)
- [Features](#features)
- [Installing](#installing)
- [Building from source](#building-from-source)
- [Copyright and license](#copyright-and-license)

<!-- /TOC -->

## Why

OpenZFS is CDDL, which is incompatible with GPLv2 and is the reason ZFS has never shipped in the mainline Linux kernel tree. This project shares **no** code with OpenZFS. It is an independent work built from the published on-disk format and behavioral observation, licensed GPL-2.0-or-later.

Because the point of the relicense is that the code is genuinely not derived from CDDL source, development runs as a formal clean room:

- **Two roles, physically separated.** A spec side may study OpenZFS and writes only fact-only functional specifications; the implementation side works from approved specifications and never sees the source. The two sides run on isolated systems with no network or filesystem path between them.
- **One crossing point, logged.** Specifications flow in through a gatekept, commit-pinned spec repo ([`zfs-gpl-spec`](https://github.com/t00mietum/zfs-gpl-spec)); questions flow back only through a written request channel in that repo. Nothing else crosses, in either direction, and both directions live permanently in git history.
- **Measured, not just promised.** A recurring spec-side audit compares this tree against the OpenZFS corpus (identifiers, string literals, comments, constant tables) so non-copying is demonstrated rather than merely asserted, and working transcripts are retained in append-only, hash-manifested archives.

The full rules and rationale - including a candid treatment of the questions AI-assisted development raises for clean rooms - are in [`cleanroom.md`](cleanroom.md); provenance is in [`provenance.md`](provenance.md). Contributors read [`contributing.md`](contributing.md) first.

Status: pre-alpha. Nothing works yet. This tree currently holds the workspace skeleton, the design, and the clean-room process that governs how the code may be written.

## Features

- On-disk compatible with OpenZFS, both directions, targeting one prior stable minor release or three months, whichever is longer. Version naming follows OpenZFS.
- Coexists with an installed OpenZFS: distinct binary and module names (`zgpl`, not `zpool`/`zfs`), no shared device paths or state files.
- Native OS page cache instead of a reimplemented ARC.
- Device discovery by native enumeration and label scan, not a cache file.

Full design in [`project/design.md`](project/design.md); roadmap in [`project/backlog.md`](project/backlog.md).

## Installing

Nothing to install yet (pre-alpha).

## Building from source

Rust workspace at the repository root:

	cargo build --release        # binary: zgpl
	cargo test
	cargo fmt --all

The approved format specifications ([`zfs-gpl-spec`](https://github.com/t00mietum/zfs-gpl-spec)) are consumed as a git submodule at `spec/`, pinned by commit. After cloning:

	git submodule update --init

## Copyright and license

> Copyright © 2026 t00mietum (ID: f⍒Ê🝅ĜᛎỹqFẅ▿⍢Ŷ‡ʬẼᛏ🜣)<br>
> Licensed under [GNU GPL v2 Or Later License](https://spdx.org/licenses/GPL-2.0-or-later.html) license. No warranty. See [`license.md`](license.md).

This is not legal advice; the clean-room and patent notes in this repo are an engineering process, pending review by counsel.
