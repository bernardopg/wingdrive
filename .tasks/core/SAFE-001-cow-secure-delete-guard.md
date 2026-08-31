---
id: SAFE-001
title: "Refuse unsafe secure deletion on CoW filesystems"
status: "Done"
assignee: "jamiepine"
priority: "High"
tags: [core, filesystem, safety, btrfs]
---

## Description

Secure overwrite allocates fresh extents on btrfs, ZFS, APFS, ReFS, and bcachefs instead of replacing the original bytes. Three overwrite passes can exhaust data and metadata space while providing no secure-erasure guarantee.

## Implementation Steps

- [x] Detect the filesystem backing the deletion target
- [x] Refuse secure overwrite on known copy-on-write and unclassified filesystems
- [x] Preflight directory entries and unlink symlinks without following them
- [x] Verify focused tests and workspace compilation

## Acceptance Criteria

- Secure deletion returns an unsupported-operation error on known CoW or unclassified filesystems
- Directory deletion refuses nested CoW mounts before overwriting any entry
- Symlinks are unlinked without overwriting files outside the selected tree
- Permanent deletion remains available
- Overwrite-in-place filesystems retain the existing secure-delete behavior
