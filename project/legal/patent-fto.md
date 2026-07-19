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

## To do

- 🔘 Engage a patent attorney for an FTO search.
- 🔘 Enumerate live vs expired Sun/Oracle ZFS patents with real dates.
- 🔘 Decide, per still-live patent that reads on an intended technique: expiry, license, design-around, or defer the feature.
