---
id: DESK-020
title: On-demand file checksum in the Inspector
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S03
tags: [desktop, inspector, core, file-operations]
last_updated: 2026-10-15
---

## Description

Checking a download against a published hash means leaving the file manager for `sha256sum`.
Dolphin and Nautilus show checksums in Properties. WingDrive's content identity is a sampled BLAKE3
`cas_id`, not a full-file hash, so it cannot be compared with published checksums.

## Implementation Steps

1. Add a core query `files.checksum` that streams one local file and returns its SHA-256 hex digest,
   registered with `register_query!`; regenerate the TypeScript types.
2. In the Inspector, show a "Calculate SHA-256" button for a single file; show the digest with a copy
   button, and a field to paste an expected hash that shows match or mismatch.
3. Run the hash off the async runtime (`spawn_blocking`) so large files do not block the daemon.
4. Rust test against a known digest; add the strings to `en` and `pt`.

## Acceptance Criteria

- [ ] The digest matches `sha256sum` for small and multi-gigabyte files
- [ ] Pasting an expected hash shows whether it matches
- [ ] The daemon stays responsive while a large file is hashed
- [ ] Verified on the Linux desktop build
