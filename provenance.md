# Provenance

This file records where the code in this repository comes from, so that its independence from CDDL-licensed ZFS source is documented rather than merely asserted. It is the public-facing summary; the working process that backs it up is in [`cleanroom.md`](cleanroom.md).

## Statement of independent creation

zfs-gpl is an original work. It shares no source code with OpenZFS, illumos, OpenSolaris, or any other CDDL-licensed ZFS implementation. Its understanding of the ZFS on-disk format is drawn from:

- The published *ZFS On-Disk Format* specification (Sun Microsystems, 2006), a document, not source code. It covers the durable core: vdev labels, the uberblock, block pointers and DVAs, the DMU object model, the DSL, the ZAP, and the ZPL.
- Public, non-source documentation: OpenZFS man pages (notably `zpool-features(7)`), the OpenZFS docs site, academic and forensic papers on the format.
- Behavioral observation: creating pools with a stock OpenZFS install and observing the resulting on-disk bytes and import/read behavior as a black box (an external oracle), without reading OpenZFS source.
- For format features introduced after 2006 that are documented only in CDDL source (feature flags, native encryption, dRAID, zstd, log spacemaps, large dnodes and blocks), functional specifications produced under the clean-room process in [`cleanroom.md`](cleanroom.md): a separate role studies the source and writes fact-only specifications; this implementation is written from those specifications, never from the source.

## Authorship

The code is written by Claude (Anthropic's LLM-based coding agent), operating as the implementation side of the clean-room process, under the direction of the human maintainer, who contributes documentation and project management rather than code. This is stated openly because the record is stronger for it: from 2026-07-19 forward, every Claude commit is authored as `Claude <claude@bubblesnet.com>` and carries trailers recording the exact model, effort setting, and isolated instance that produced it (`Model:` / `Effort:` / `Instance:`). The `Instance:` trailer distinguishes the implementation side from the spec side - part of the evidence that the clean-room roles are distinct actors. Commits before that date were authored under the maintainer's project identity; history is not rewritten.

## What is deliberately absent

- No file copied or adapted from any CDDL-licensed tree.
- No OpenZFS `AUTHORS`, comments, identifier schemes, or CDDL headers.
- No OpenZFS test-suite source. Conformance is checked with independently authored tests and by round-tripping against a stock OpenZFS binary as an external oracle. See [`cleanroom.md`](cleanroom.md) on tests.

## The two open questions this does not resolve

Clean-room process addresses **copyright** only. Two separate risks are tracked, not solved, here and require review by counsel:

- **Patents.** Independent creation is no defense to a patent. The status of the original Sun/Oracle ZFS patents and any third-party filesystem patents is a freedom-to-operate question. See [`project/legal/patent-fto.md`](project/legal/patent-fto.md).
- **The forfeited CDDL patent grant.** OpenZFS distributors receive CDDL's built-in patent license; by sharing no CDDL code, this project does not. That trade-off is deliberate but should be weighed with counsel.

Nothing in this repository is legal advice.
