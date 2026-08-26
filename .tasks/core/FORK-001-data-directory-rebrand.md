---
id: FORK-001
title: "Migrate data directories and keyring service to WingDrive"
status: "Done"
assignee: bernardopg
priority: "High"
tags: ["fork", "core", "migration"]
last_updated: 2026-08-26
---

## Description

The WingDrive rebrand renamed the user-facing product identity. On-disk paths
still had to keep working for installs predating the fork, otherwise a user
upgrading would see an empty app and unreachable encryption keys.

`core::branding` now owns every on-disk name. Each resolver prefers the
WingDrive location and falls back to the legacy `Spacedrive` one only when it
already exists. Nothing is moved or deleted, so the original Spacedrive build
keeps working against the same data.

Existence on disk is the tiebreaker rather than a stored migration flag,
because a flag desyncs from reality when a home directory is copied between
machines.

## Implementation Steps

- [x] Add `core/src/branding.rs` owning the app name, keyring service and path resolvers
- [x] Resolve data, config and library directories with a legacy fallback
- [x] Route `config::default_data_dir`, `DeviceConfig::config_path` and `LibraryManager` through it
- [x] Adopt a pre-fork device key from the `Spacedrive` keyring service on first run
- [x] Rebrand user-facing log lines and CLI output
- [x] Cover upgrade-from-Spacedrive in an integration test

## Deliberately Not Done

- The `spacedrive.json` config filename is unchanged. It lives inside the
  resolved data directory, is never shown to users, and renaming it would break
  the fallback for zero benefit.
- Legacy keyring and directory entries are never deleted. Removing them would
  make a rollback to the original build destructive.

## Acceptance Criteria

- [x] A fresh install writes only to WingDrive paths.
- [x] An existing Spacedrive install keeps its libraries, keys and device identity.
- [x] No silent data loss when the keyring entry is missing; the failure is explicit and actionable.

## Validation

- `cargo test -p sd-core --lib branding::` covers the four resolution cases and
  legacy path detection. 7 tests pass.
- `cargo test -p sd-core --test branding_legacy_paths` overrides `$HOME` and the
  XDG variables to assert legacy adoption, fresh-install behaviour, and that a
  completed migration stops falling back. This test caught `dirs::config_dir`
  reading `XDG_CONFIG_HOME` instead of `$HOME`.
- The daemon was started against the real machine and logged
  `Initializing WingDrive at "/home/bitter/.spacedrive"`, then opened the
  existing `My Library.sdlibrary` and its database.

## Follow-up

Using a separate data directory for development builds is still open and
tracked in `TODO` under core work.
