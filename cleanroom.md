# Clean-room process

This is the process that keeps zfs-gpl an independent work rather than a derivative of CDDL-licensed ZFS. It exists to produce, as we go, believable evidence that the implementation was created without copying OpenZFS source. That evidence is what lets us license the result GPL-2.0-or-later at all.

Nothing here is legal advice. It is an engineering process derived from the established clean-room case law (the Phoenix/Compaq BIOS reimplementation, NEC v. Intel, Sega v. Accolade, Sony v. Connectix, and Google v. Oracle). It is pending review by counsel, and the patent track below is explicitly out of its scope.

## The principle in one paragraph

Copyright protects expression, not facts, formats, methods, or interfaces. An on-disk format is fact and interface. Reimplementing it is legal; the only real risk is copying the *expression* of the OpenZFS source that implements it. Clean-room process does not make copying legal, it makes non-copying provable, by ensuring the people (or agents) who write the code never had access to the source they might otherwise be accused of copying.

## The two roles, and the wall between them

**Spec side (may look).** May read, run, disassemble, and inspect OpenZFS source and a live OpenZFS install. Produces exactly one kind of output: functional specifications, in its own words, describing *what* the format and behavior are. Never writes implementation code for this project.

**Implementation side (may not look).** Works only in this repo. Reads only approved specifications. Never opens OpenZFS source, never runs it to read its internals, never has it mounted or indexed or in context. Everything it writes is presumptively independent because it had no access to the thing it could be accused of copying.

**Gatekeeper (guards the wall).** Reviews every specification before it crosses from spec side to implementation side, and strips anything that leaks expression rather than fact. Logs what crossed, authored by whom, approved when.

The wall is enforced by tooling, not by good intentions: separate trees, separate repos, separate agent contexts. See "Layout" below.

## What may cross the wall

Facts and interface, which copyright does not protect:

- On-disk layout: label and boot block structure, the uberblock, block-pointer and DVA encoding, gang and indirect blocks, dnode and object-set structures, the DSL layout, ZAP micro and fat forms, ZIL records, endianness, alignment and padding, the Merkle-tree scheme.
- Magic numbers, version numbers, feature GUIDs and their semantics, checksum and compression algorithm identities and parameters.
- Behavioral descriptions stated as requirements: "on import, select the active uberblock as the one with the highest txg whose checksum verifies."
- Test cases stated as input/expected-output facts.
- Separately specified public algorithms (fletcher4, SHA-256, LZ4, and so on), which carry their own provenance and licensing.

## What may never cross the wall

Expression, which copyright does protect:

- Source code, verbatim or paraphrased.
- Structure, sequence, and organization where it was an authorial choice rather than dictated by the format: the particular decomposition into functions, the call graph, module boundaries, the ordering of non-mandated helpers.
- Comments, documentation prose, identifier names taken from the source.
- Error-message text and other arbitrary expressive choices.

The gatekeeper's test: a sentence that says "the format requires X" is fact and may cross. A sentence that says "the code does X by calling A then B then C" is structure and is stripped, unless that order is functionally mandated by the format.

## Layout that enforces the wall

- **Dirty tree** (outside this repo, never published by us): a clone of upstream `openzfs/zfs` plus the spec side's raw analysis notes. It lives in a separate filesystem tree with its own working context. This repo never references it, and `.gitignore` defensively blocks anything from it.
- **Spec repo** (separate GitHub repo): the gatekeeper-approved functional specifications, and only those. Its commit history is authored by the spec/gatekeeper identity, which proves the roles are distinct actors. This is the only channel into implementation.
- **Implementation repo** (this one): the GPL code. It consumes the spec repo pinned to specific commit hashes, and its history shows it only ever consumed spec, never source.

## Working as agents

The single-actor rule applies identically to an automated agent: one agent context that ingests OpenZFS source and also emits implementation code destroys the defense, and does so on the record, because the transcript logs exactly what it read. So:

- The spec-side agent has tool access to the dirty tree and can write only into the spec repo. It cannot write to this repo.
- The implementation-side agent has access to the spec and this repo only. The dirty tree is not mounted, not indexed, not fetchable from its context.
- No single agent context ever spans both sides. A question that can only be answered from the source goes back through spec side and the gatekeeper, never directly to implementation.
- Transcripts and tool logs are the evidence trail. They are kept, because they affirmatively show the implementation side never saw the source. They are also discoverable, so the discipline has to be real.

## Evidence to keep

- Which identities are spec side vs implementation side, and when.
- The specifications, versioned, each showing author, gatekeeper approval, and date.
- The gatekeeper's redaction/approval log.
- Agent transcripts and tool logs for both sides.
- Periodic clone-detection scans of this tree against the OpenZFS corpus, run as a tripwire (not as guidance), with what was found and what was done about it.
- Provenance for every third-party algorithm and its license.

## Tests

Test *code* is copyrightable expression, so no OpenZFS test-suite source enters this tree. Test *cases* are facts and may be derived: "a pool at feature set F, given these writes, must import and yield this checksum" is a statement about the format. Prefer independently authored conformance tests plus round-tripping against a stock OpenZFS binary used as an external oracle (running the real tool and comparing behavior is black-box observation, not copying). Do not clone the organization of the OpenZFS test suite even where individual cases are facts.

## Out of scope: patents

This process defeats copyright claims. It does nothing about patents, which independent creation cannot cure. That is a separate, counsel-led freedom-to-operate track. See [`project/legal/patent-fto.md`](project/legal/patent-fto.md).
