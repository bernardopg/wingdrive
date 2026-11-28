---
id: ADAPT-029
title: "Registry CI builds compiled adapters from the pinned commit"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: Medium
milestone: M5
sprint: S09
tags: [adapters, store, ci, security]
depends_on: [ADAPT-007, ADAPT-008]
related_tasks: [ADAPT-001]
last_updated: 2026-11-28
---

## Description

Moved out of ADAPT-007, whose other checks are done. Adapters with `runtime.kind = "binary"` must be
built by the registry's own CI from the reviewed commit, so what users install always matches the
reviewed source; binaries uploaded by authors are never accepted. Today `wingreg` refuses every
binary adapter ("compiled adapters are built by build.yml, which is not enabled yet"), and there is
no Rust template to build one from (ADAPT-008).

## Implementation Steps

- [ ] `build.yml` in the registry: for each binary adapter in a pull request or publish, check out the pinned commit and build it for Linux x86_64 and aarch64, macOS aarch64 and Windows x86_64 with `cargo build --locked --release`, no network after `cargo fetch`
- [ ] Refuse a submission that commits a binary or has no `Cargo.lock`; audit its dependencies (`cargo deny` or `cargo audit`)
- [ ] The index gets one archive and SHA-256 per target; the app installs the archive for its own target and refuses an adapter with none
- [ ] Tests and the contract harness run on the built binary, as for Python adapters
- [ ] `wingreg` stops refusing `runtime.kind = "binary"` once the build passes
- [ ] The Rust template from ADAPT-008 is the first adapter built this way

## Acceptance Criteria

- [ ] A binary adapter from the Rust template is built, signed into the index and installed from the store on Linux, macOS and Windows
- [ ] A submission with a committed binary or a missing lock file fails review with a message that says why
