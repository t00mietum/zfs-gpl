<!-- omit in toc -->
# Style guide

The canonical style reference for this project - prose, comments, code, and commits. It is short on purpose. Where a language's own idioms or its standard formatter conflict with anything here, the idiom wins; this guide covers the choices the tools leave open.

Deep Rust conventions (the data-oriented paradigm, parse-don't-validate, the anti-LLM-tendency rules) live in [`project/design.md`](project/design.md) under "Rust paradigm and conventions" and are not repeated here.

<!-- omit in toc -->
## Contents

- [Prose and documentation](#prose-and-documentation)
- [Comments](#comments)
- [Code](#code)
- [Rust](#rust)
- [File headers and licensing](#file-headers-and-licensing)
- [Commit messages](#commit-messages)

## Prose and documentation

Write for a reader who is skimming.

- Never hard-wrap. One paragraph or bullet is one physical line; let the editor wrap it. Reserve newlines for real structure - paragraph breaks, list items, nesting, code blocks.
- Prefer short sentences. Break a complex idea into a few nested bullets rather than one long sentence joined by dashes, semicolons, and parentheticals.
- Go easy on emphasis. Bold, italics, and ALL-CAPS are for the rare word that genuinely needs it, not for color. The same goes for dramatic adjectives and adverbs.
- ASCII only. Use `->` not an arrow glyph, `-` not an en- or em-dash. The one exception is `(C)`: write `©` where a copyright symbol is wanted.
- Filenames are lower-case (`contributing.md`, `changelog.md`), with `README.md` the deliberate exception.
- Indent with tabs, align with spaces.

## Comments

- Terse. Explain why, not what. A comment that restates the next line is noise.
- No narration ("This function does X"), no banner dividers, no flowerboxing. Where a genuinely critical section needs a visual break, use the plain industry style for that language, nothing custom.
- ASCII only, same as prose. Document Unicode only when the thing being documented is itself Unicode.

## Code

- Indent with tabs; align with spaces. This holds wherever the language and its primary formatter allow it.
- Name things so a human can find them. `upperBound` is easier to read and grep than `ub`. The goal is "easy to locate what you mean", not maximum length.
	- Do not overcorrect. Short conventional names are fine where they are clear: loop counters (`i`), a local `err`, an index in a short combinator.
	- The rule targets short, random, or misleading names on variables that carry real meaning - not idiomatic throwaways.
- Return early. Keep the happy path at minimum indentation with guard clauses instead of deep nesting.
- Prefer the standard library and its idioms over a hand-rolled equivalent or a pulled-in dependency, as long as it stays readable.

## Rust

The full paradigm is in [`project/design.md`](project/design.md). The load-bearing points:

- Format with `rustfmt` as configured. This repo pins `hard_tabs = true`, so Rust indents with tabs like everything else. Do not "fix" it to spaces - the pin overrides rustfmt's space default on purpose.
- Write to pass `clippy::pedantic`. It is enabled as a workspace lint; CI treats warnings as errors.
- Errors are values. Return `Result`, propagate with `?`, and never `unwrap`/`expect`/panic on bytes read from disk. Use `thiserror` in the library crates and `anyhow` at the application edge.
- Derive rather than hand-roll, and derive `Debug` on every public type.
- Reach for borrows before clones, ownership before `Arc<Mutex>`, and a concrete type before a premature abstraction.

## File headers and licensing

- Every source file carries an SPDX header. Project code is `GPL-2.0-or-later`.
- Standalone helper and utility scripts are usually MIT, regardless of the project's license - license the script for what it is.
- Do not bring in a file under an incompatible license, and never copy CDDL ZFS source or its identifiers, comments, or tests. See [`cleanroom.md`](cleanroom.md).

## Commit messages

- The subject line is short, high-level, and factual - a label, not a summary of the diff ("Updated", "label checksum"). Real detail belongs in the code and the project notes, not the message.
- Attribution trailers (model, effort, instance, co-author) are required on every commit. The mechanics are in [`contributing.md`](contributing.md) and the project's `CLAUDE.md`.
