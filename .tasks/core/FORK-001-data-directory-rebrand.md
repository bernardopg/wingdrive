---
id: FORK-001
title: "Migrate data directories and keyring service to WingDrive"
status: "To Do"
assignee: bernardopg
priority: "High"
tags: ["fork", "core", "migration", "breaking"]
last_updated: 2026-08-24
---

## Description

The WingDrive rebrand renamed the user-facing product identity (bundle
identifier, binary, window title, macOS menu and LaunchAgent label) but left
on-disk paths untouched. Renaming those in the same commit would orphan every
existing library without a migration path, so it is tracked separately.

Remaining `Spacedrive` identifiers on disk:

- `core/src/config/mod.rs` - config and data directories (`spacedrive`)
- `core/src/device/config.rs` - platform dirs (`com.spacedrive`, `spacedrive`, `Spacedrive`)
- `core/src/library/manager.rs` - library search paths (`~/Spacedrive/Libraries`)
- `core/src/crypto/key_manager.rs` - `KEYRING_SERVICE = "Spacedrive"`

The keyring entry is the sharpest edge. Renaming the service makes existing
encryption keys unreachable, which reads as data loss even though the files are
intact.

## Implementation Steps

- [ ] Resolve the WingDrive directory, falling back to the legacy path when it exists
- [ ] Migrate the keyring entry from service `Spacedrive` to `WingDrive` on first run
- [ ] Move or symlink existing libraries after a confirmed user prompt
- [ ] Use a distinct data directory for development builds
- [ ] Cover upgrade-from-Spacedrive in an integration test

## Acceptance Criteria

- A fresh install writes only to WingDrive paths.
- An existing Spacedrive install keeps its libraries, keys and device identity.
- No silent data loss when the keyring entry is missing; the failure is explicit and actionable.
