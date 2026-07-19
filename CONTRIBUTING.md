# Contributing

Before anything else: this project's value depends on staying clean-room. Read [`CLEANROOM.md`](CLEANROOM.md) and take it literally. A single tainted contribution can force a file back to CDDL and break the GPL relicense for that file, so the bar is not "don't copy code" but "don't let CDDL expression into your head and then into the tree."

## The one rule

If you have read OpenZFS (or any CDDL ZFS) source, you may not write implementation code here. The two roles are mutually exclusive:

- **Spec side** may read the source and writes only functional specifications (facts, formats, behaviors) into the spec repo. Never implementation code.
- **Implementation side** works in this repo, from the spec only, and never opens OpenZFS source.

Pick one per feature area and stay on that side.

## Sign-off

Every commit must carry a Developer Certificate of Origin sign-off plus a clean-room attestation:

    Signed-off-by: Real Name <email>

By signing off you additionally attest that the contribution contains no code copied or derived from OpenZFS, illumos, OpenSolaris, or any other CDDL-licensed source, and that if you authored it as implementation code you did so without consulting such source.

## Style

- Rust is rustfmt-canonical; a formatting hook runs on staged `.rs`.
- Comments explain why, not what. ASCII only. No banner dividers.
- Commit messages are short and factual.

## What not to bring in

- No CDDL-licensed file, "for reference" or otherwise. The reference source lives in a separate tree that is never part of this repo.
- No OpenZFS comments, identifier names, or test-suite source.
- License everything you add GPL-2.0-or-later, with an SPDX header.
