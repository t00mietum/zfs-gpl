<!-- omit in toc -->
# Contributing

Thanks for taking the time to contribute.

Before anything else: this project's value depends on staying clean-room. Read [`cleanroom.md`](cleanroom.md) and take it literally. A single tainted contribution can force a file back to CDDL and break the GPL relicense for that file, so the bar is not "don't copy code" but "don't let CDDL expression into your head and then into the tree."

<!-- omit in toc -->
## Table of Contents

- [The one rule (clean-room)](#the-one-rule-clean-room)
- [Sign-off](#sign-off)
- [Code of Conduct](#code-of-conduct)
- [I Have a Question](#i-have-a-question)
- [I Want To Contribute](#i-want-to-contribute)
	- [Reporting Bugs](#reporting-bugs)
	- [Suggesting Enhancements](#suggesting-enhancements)
- [Style](#style)


## The one rule (clean-room)

If you have read OpenZFS (or any CDDL ZFS) source, you may not write implementation code here. The two roles are mutually exclusive:

- **Spec side** may read the source and writes only functional specifications (facts, formats, behaviors) into the spec repo. Never implementation code.
- **Implementation side** works in this repo, from the spec only, and never opens OpenZFS source.

Pick one per feature area and stay on that side. Details and the legal basis are in [`cleanroom.md`](cleanroom.md).

What not to bring in:

- No CDDL-licensed file, "for reference" or otherwise. The reference source lives in a separate tree that is never part of this repo.
- No OpenZFS comments, identifier names, or test-suite source.
- License everything you add GPL-2.0-or-later, with an SPDX header.


## Sign-off

Every commit must carry a Developer Certificate of Origin sign-off plus a clean-room attestation:

	Signed-off-by: Real Name <email>

By signing off you additionally attest that the contribution contains no code copied or derived from OpenZFS, illumos, OpenSolaris, or any other CDDL-licensed source, and that if you authored it as implementation code you did so without consulting such source.


## Code of Conduct

This project and everyone participating in it is governed by the [Code of Conduct](code_of_conduct.md). By participating, you are expected to uphold this code.


## I Have a Question

Before you ask, search the existing [Issues](https://github.com/t00mietum/zfs-gpl/issues) and the wider web. If you still need clarification:

- Open an [Issue](https://github.com/t00mietum/zfs-gpl/issues/new).
- Provide as much context as you can about what you're running into.
- Provide project and platform versions, depending on what seems relevant.


## I Want To Contribute

> ### Legal Notice <!-- omit in toc -->
> When contributing to this project, you must agree that you have authored 100% of the content, that you have the necessary rights to the content, and that the content you contribute may be provided under the project license. See also [The one rule](#the-one-rule-clean-room) and [Sign-off](#sign-off) above.

### Reporting Bugs

<!-- omit in toc -->
#### Before Submitting a Bug Report

- Make sure you are using the latest version.
- Determine if it is really a bug and not a local misconfiguration (read the [README](README.md); for support see [I Have a Question](#i-have-a-question)).
- Check the [bug tracker](https://github.com/t00mietum/zfs-gpl/issues?q=label%3Abug) for an existing report.
- Collect: OS, platform and version; environment versions; your input and the output; whether it reproduces reliably and on older versions; clean reproduction steps from scratch.

<!-- omit in toc -->
#### How Do I Submit a Good Bug Report?

> Never report security issues, vulnerabilities, or bugs with sensitive information in public. Instead report them privately via GitHub's [private vulnerability reporting](https://github.com/t00mietum/zfs-gpl/security/advisories/new) for this repository.

For non-sensitive bugs:

- Open an [Issue](https://github.com/t00mietum/zfs-gpl/issues/new).
- Explain the behavior you expected and the actual behavior.
- Provide reproduction steps someone else can follow, ideally a reduced test case, plus the information collected above.

### Suggesting Enhancements

<!-- omit in toc -->
#### Before Submitting an Enhancement

- Make sure you are using the latest version, and read the [README](README.md) to check the feature isn't already covered.
- [Search](https://github.com/t00mietum/zfs-gpl/issues) for an existing suggestion; if it exists, comment there instead of opening a new one.
- Consider whether it fits the scope and aims of the project.

<!-- omit in toc -->
#### How Do I Submit a Good Enhancement Suggestion?

Enhancement suggestions are tracked as [GitHub issues](https://github.com/t00mietum/zfs-gpl/issues).

- Use a clear and descriptive title.
- Give a step-by-step description of the suggested enhancement.
- Describe the current behavior, the behavior you expected instead, and why.
- Explain why the enhancement would be useful to most users.


## Style

- Rust is rustfmt-canonical; a formatting hook runs on staged `.rs`.
- Comments explain why, not what. ASCII only. No banner dividers.
- Commit messages are short and factual.
