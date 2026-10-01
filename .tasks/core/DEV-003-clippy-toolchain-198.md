---
id: DEV-003
title: Restore Workspace Clippy on Toolchain 1.98
status: To Do
assignee: bernardopg
parent: DEV-000
priority: High
milestone: M1
tags: [ci, clippy, toolchain]
last_updated: 2026-10-01
---

## Description

On 2026-10-01, `cargo clippy --workspace --locked -- -D warnings` fails at HEAD with about 91 errors in crates that recent work did not touch (`archive`, `ffmpeg`, `media-metadata`, `task-system`, `crypto`), mostly `doc_markdown` and `unused_async`. Because clippy stops at the first failing dependency, `wing-core` and `wing-cli` are not linted at all under `-D warnings`. This is the CI gate.

## Acceptance Criteria

- [ ] Confirm whether CI on `main` fails the same way
- [ ] Fix or explicitly allow the new lints per crate
- [ ] `cargo clippy --workspace --locked -- -D warnings` passes locally
