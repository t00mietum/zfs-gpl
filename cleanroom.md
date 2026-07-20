# Clean-room process

This is the process that keeps zfs-gpl an independent work rather than a derivative of CDDL-licensed ZFS. It exists to produce, as we go, believable evidence that the implementation was created without copying OpenZFS source. That evidence is what lets us license the result GPL-2.0-or-later at all.

Nothing here is legal advice. It is an engineering process derived from the established clean-room case law (the Phoenix/Compaq BIOS reimplementation, NEC v. Intel, Sega v. Accolade, Sony v. Connectix, and Google v. Oracle). It is pending review by counsel, and the patent track below is explicitly out of its scope.

This is version 2 of the process. Version 1 established the roles and the spec pipeline; version 2 adds physical isolation of the two sides, a single logged crossing point, a continuous similarity audit, an evidence-retention regime, and a candid treatment of the shared-model question.

## The principle in one paragraph

Copyright protects expression, not facts, formats, methods, or interfaces. An on-disk format is fact and interface. Reimplementing it is legal; the only real risk is copying the *expression* of the OpenZFS source that implements it. Clean-room process does not make copying legal, it makes non-copying provable, by ensuring the people (or agents) who write the code never had access to the source they might otherwise be accused of copying. And because process alone can always be doubted, this process also produces affirmative output evidence: a recurring audit showing the shipped code is not substantially similar to the source it never saw.

## The roles, and the wall between them

**Spec side (may look).** May read, run, disassemble, and inspect OpenZFS source and a live OpenZFS install. Produces exactly one kind of output: functional specifications, in its own words, describing *what* the format and behavior are. Never writes implementation code for this project.

**Implementation side (may not look).** Works only in this repo. Reads only approved specifications. Never opens OpenZFS source, never runs it to read its internals, never has it mounted or indexed or in context. Everything it writes is presumptively independent because it had no access to the thing it could be accused of copying.

**Gatekeeper (guards the wall).** Reviews every specification before it crosses from spec side to implementation side, and strips anything that leaks expression rather than fact. Logs what crossed, authored by whom, approved when. The gatekeeper operates on the spec side of the wall.

**Monitor (the maintainer).** The human maintainer administers both environments but performs neither technical role. The monitor coordinates work, reviews approved specifications (English statements of fact), and carries go/no-go decisions between the sides - never content. The monitor does not read the OpenZFS source and, having not worked in C in decades, could not meaningfully transcribe it; this is noted because it is itself evidence. Both sides being administered by one maintainer is normal for clean rooms - the historical ones were run inside single companies - and is disclosed here rather than obscured. What matters is that the *roles* never collapse into one actor.

The wall is enforced by tooling and physical separation, not by good intentions.

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

## The physical wall

The two sides run in physically separate environments with no path between them:

- **Spec environment**: a separate system (its own machine or a VM on a different host - never a VM nested on the implementation host) holding the dirty tree (the upstream `openzfs/zfs` clone, pinned), the spec repo working tree, and its own agent installation with its own instructions, memory, and settings. It has no network or filesystem visibility into the implementation environment.
- **Implementation environment**: holds this repo and the approved spec repo at a pinned commit, and nothing else. No copy of OpenZFS source exists on it, readable or otherwise. "Could not have read it" is the standard here, not "promised not to read it". As defense in depth, the implementation-side tooling additionally carries deny rules for the dirty-tree paths, so an accidental reappearance of a source copy is refused and the refusal is logged.
- The environments share no login session, no agent installation, no working memory, and no mounted filesystem. The only things both can reach are the published git repositories.

## The single crossing point

Exactly one artifact crosses the wall, in one direction: the gatekeeper-approved spec repo, consumed by this repo pinned to specific commit hashes.

The reverse direction - the implementation side needs a format area specified - goes through a logged request channel: `requests.md` in the spec repo. Requests are questions and functional needs only ("specify the fat ZAP hash and collision behavior"); they never contain implementation code and never ask for source structure. Answers arrive only as approved specifications. Every request and its disposition is permanently logged in the spec repo's history, which turns the question channel from a risk into evidence. Historical clean rooms worked the same way: questions in writing through the monitor, answers in writing, everything logged.

## Working as agents

Both sides use LLM-based agents. The single-actor rule applies identically to an automated agent: one agent context that ingests OpenZFS source and also emits implementation code destroys the defense, and does so on the record, because the transcript logs exactly what it read. So:

- Spec-side agents (analysis and gatekeeper) run only in the spec environment. Implementation-side agents run only in this one. No agent context ever spans both sides, and neither side spawns or orchestrates agents of the other - the implementation side only ever pulls the published spec repo.
- A question that can only be answered from the source goes through `requests.md` and the gatekeeper, never directly to an implementation context.
- Transcripts and tool logs are the evidence trail. They are retained under the regime below, because they affirmatively show the implementation side never saw the source. They are also discoverable, so the discipline has to be real.

## The shared-model question

Both sides run on the same commercial large language model, and that model's training corpus almost certainly includes OpenZFS source. This is stated plainly because it is the honest novel question in an AI-era clean room, and no case law settles it yet. The process treats it as follows:

- Agent sessions are isolated: separate sessions share no context or memory, and that isolation is documentable. The classic wall assumes two teams' brains cannot share memories; two sessions genuinely do not.
- The exposure is not unique to this project - it is the baseline condition of all AI-assisted software, including most commercial development today. The remedy is the same for everyone: demonstrate that the output does not reproduce protected expression.
- So the process does not rest on the model's ignorance. It rests on (a) the input channel: implementation contexts receive facts only, through the gatekept spec pipeline, and their transcripts prove it; (b) the output audit below, which affirmatively measures the result against the source; and (c) structural divergence: a different language (Rust, not C), a different architecture (native OS caching rather than an ARC port, native device discovery), and module boundaries driven by the spec rather than by the source's decomposition.

## Continuous similarity audit

A recurring audit compares this repo against the pinned OpenZFS corpus and logs the result. It is a tripwire, not guidance: findings are removed and root-caused, never "fixed toward" the source.

- It runs on the spec side (it must read the source; the published implementation repo it also reads is public, so this contaminates nothing) and reports only pass/fail plus metrics.
- Cross-language structural comparison (C to Rust) is weak, but leakage through a defective spec is lexical, so the audit harvests exactly that: identifiers, string literals, comments, error messages, and constant tables, diffed against the same harvest of the source.
- Interface facts that are legitimately identical (magic numbers, on-disk key strings, feature GUIDs) are allowlisted from the approved specs themselves, so a hit means something actually leaked.
- Results are recorded in the spec repo's gatekeeper log. A long record of green audits is the affirmative half of the defense.

## Evidence to keep

- Which identities and environments are spec side vs implementation side, and when.
- The specifications, versioned, each showing author, gatekeeper approval, and date.
- The gatekeeper's redaction/approval log, and the request log (`requests.md` history).
- Agent transcripts and tool logs for both sides: retention windows raised well beyond tool defaults, mirrored into append-only archives with a SHA-256 manifest, the manifest periodically anchored to a third-party timestamp (a hosted commit or a public timestamping service). Archives are append-only on principle: evidence is never rotated away.
- The similarity audit reports, including any findings and their remediation.
- The monitor's dated log of what ran and what crossed.
- Provenance for every third-party algorithm and its license.

## Tests

Test *code* is copyrightable expression, so no OpenZFS test-suite source enters this tree. Test *cases* are facts and may be derived: "a pool at feature set F, given these writes, must import and yield this checksum" is a statement about the format. Prefer independently authored conformance tests plus round-tripping against a stock OpenZFS binary used as an external oracle (running the real tool and comparing behavior is black-box observation, not copying). Do not clone the organization of the OpenZFS test suite even where individual cases are facts.

## Out of scope: patents

This process defeats copyright claims. It does nothing about patents, which independent creation cannot cure. That is a separate, counsel-led freedom-to-operate track. See [`project/legal/patent-fto.md`](project/legal/patent-fto.md).
