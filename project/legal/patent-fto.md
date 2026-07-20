# Patent freedom-to-operate notes

Placeholder for the patent track. This is out of scope for the clean-room process: independent creation defeats copyright claims, not patents. You can infringe a patent you never heard of, having written every line yourself. This needs a patent attorney; the notes below are only to frame that engagement.

Nothing here is legal advice.

## Why this is separate

- Clean-room proves non-copying. It says nothing about whether a technique is patented.
- So a freedom-to-operate (FTO) search runs in parallel with implementation, not as part of it.

## What to search

- Sun/Oracle ZFS patents. Filed roughly 2001-2006 during ZFS development; US utility patents run 20 years from earliest non-provisional filing, so many are expired or expiring around 2021-2026. Continuations and divisionals can carry later dates. Each needs its actual priority date, expiration (with term adjustments and maintenance-fee status), and a claim-by-claim read against what we intend to implement.
- Third-party filesystem patents, not just Oracle's. The NetApp v. Sun dispute (2007-2010, over WAFL copy-on-write and snapshot patents) settled with no ruling; some of those patents have since expired, but COW/snapshot techniques are worth clearing against third parties generally.

## The CDDL patent-grant trade-off

- CDDL carries an express patent license from Sun/Oracle and contributors, but only to licensees of the CDDL code.
- By sharing no CDDL code, this project is not a CDDL licensee and does not get that grant.
- So the pure-GPL clean-room path maximizes copyright safety while forfeiting the patent peace OpenZFS distributors enjoy. Deliberate, but weigh it with counsel.

## Preliminary enumeration (2026-07-20, NOT attorney-grade)

Bibliographic-only inventory: numbers, dates, and legal status read off each patent's Google Patents page. This is not a claim read and not legal advice; it exists to scope the counsel engagement, and two known gaps remain (see end). "Est. expiration" is the adjusted/anticipated date Google Patents shows (reflects any PTA), which for several 2005-2006-priority patents runs later than a naive filing+20.

### Expired - clear to practice (copyright aside)

Third-party NetApp WAFL patents (the 2007-2010 cross-suit set) - all expired:

| Patent | Title (short) | Owner | Est. expiration |
|---|---|---|---|
| US 6,892,211 | COW consistency and block usage | NetApp | 2013-06-03 |
| US 5,963,962 | Write Anywhere File Layout (WAFL) | NetApp | 2015-05-31 |
| US 7,174,352 | File system image transfer | NetApp | 2015-06-07 |
| US 5,819,292 | Consistent states / user-accessible read-only copies | NetApp | 2015-10-06 |
| US 6,857,001 | Multiple concurrent active file systems | NetApp | 2022-09-27 |
| US 7,162,486 | Named data streams in an on-disk structure | NetApp | 2022-12-25 |

Foundational Sun/Oracle ZFS mechanics (2004-filed core) - all now expired:

| Patent | Title (short) | Owner | Est. expiration |
|---|---|---|---|
| US 7,424,574 | Dynamic striping (variable stripe width) | Oracle | 2024-11-30 |
| US 7,412,450 | Tamper detection / hierarchical checksum (Merkle) | Oracle | 2025-10-14 |
| US 7,533,225 | Adaptive endianness | Oracle | 2026-01-31 |
| US 7,415,653 | Vectored block-level checksum (end-to-end integrity) | Oracle | 2026-04-02 |

### Live - still in force (Oracle America / Oracle Intl, unless noted)

| Patent | Title (short) | Reads on | Est. expiration |
|---|---|---|---|
| US 7,689,877 | Using checksums to repair data | self-healing | 2028-07-10 |
| US 7,743,225 | Ditto blocks | redundant metadata copies | 2029-04-22 |
| US 7,925,827 | Dirty time logging | resilver bookkeeping | 2029-09-23 |
| US 8,938,594 | Metadata-based resilvering | resilver | 2029-10-30 |
| US 7,865,673 | Multiple replication levels with pooled devices | per-file/dataset redundancy | 2029-11-04 |
| US 8,218,759 | System and method for encrypting data | native encryption | 2030-10-28 |
| US 8,280,858 | Storage pool scrubbing with concurrent snapshots | scrub | 2031-04-03 |
| US 8,549,051 | Unlimited file system snapshots and clones | snapshots/clones | 2031-09-17 |
| US 9,215,066 | Making info in a COW file system inaccessible | secure delete | 2031-11-08 |
| US 9,742,564 | Method and system for encrypting data | native encryption | 2032-10-30 |
| US 11,334,528 | ZFS block-level dedup at cloud scale | ZFS appliance cloud | 2037-08-26 |
| US 10,657,167 | Cloud gateway for ZFS snapshot storage | ZFS appliance cloud | 2037-09-27 |
| US 10,540,384 | Compressed, E2E-encrypted ZFS cloud storage | ZFS appliance cloud | 2037-10-20 |

### Reading of the above

- The COW/snapshot filesystem concept is clear: every NetApp WAFL patent that made ZFS contentious in 2007-2010 has expired (latest 2022), and the foundational ZFS mechanics (pooled dynamic striping, Merkle checksums, adaptive endianness, block checksums) expired between late 2024 and April 2026. A read-only on-disk-compatible importer sits on the friendliest side of this.
- The live cluster (2028-2032) is the differentiating feature set: self-healing, ditto blocks, resilvering, per-dataset redundancy, snapshots/clones, scrub, native encryption, secure delete. A writing / self-healing / snapshotting reimplementation must clear these feature-by-feature - several map directly onto the roadmap.
- The three 2037 patents are Oracle ZFS Storage Appliance cloud-gateway plumbing, not local on-disk format - most likely not implicated by a local reimplementation, but confirm scope.

### Known gaps (close before relying on this)

- No distinctly-titled RAID-Z parity / reconstruction grant was pinned to a verified page; the variable-stripe mechanism appears split across US 7,424,574 and US 7,865,673.
- No single foundational COW / uberblock / pooled-storage grant was confirmed under a clean title.
- Both need an assignee search (Sun Microsystems + Bonwick/Moore/Ahrens) cross-checked against Oracle's own ZFS patent list - attorney territory.

## To do

- 🔘 Engage a patent attorney for an FTO search.
- 🛠️ Enumerate live vs expired Sun/Oracle ZFS patents with real dates. Preliminary bibliographic inventory above (2026-07-20); two gaps remain (RAID-Z parity, foundational COW grant) plus attorney-grade claim reads.
- 🔘 Decide, per still-live patent that reads on an intended technique: expiry, license, design-around, or defer the feature.
