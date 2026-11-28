---
id: ADAPT-004
title: "Store backend in the daemon: install, update, remove and lockfile"
status: In Progress
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, store, core, security]
depends_on: [ADAPT-001, ADAPT-003]
related_tasks: [ADAPT-005, ADAPT-009]
last_updated: 2026-11-28
---

## Description

The daemon fetches the signed index, verifies it, and installs adapters into the data directory, so
every library can use them. Installs are pinned and recorded in a lockfile, like
`dms plugins lock`, so a library restored on another device gets the same revisions.

## Implementation Steps

- [x] Fetch and cache the index; verify the signature with the bundled public key before parsing; keep the last good index for offline use
- [x] Install: download the archive, check its SHA-256 against the index, unpack into a staging directory, validate the manifest and contract version, then move it into place atomically
- [ ] Update when the index pins a new commit; remove; reinstall from the lockfile
- [ ] Show the permissions an adapter asks for and require consent before install and when an update asks for more
- [x] Operations `adapters.store.list`, `adapters.store.install`, `adapters.store.update`, `adapters.store.remove` and `adapters.store.refresh`, with TypeScript types generated
- [ ] `wing adapters browse|install|update|remove|lock|restore`
- [x] Report a missing runtime (no `python3`) on the card instead of failing a sync

## Acceptance Criteria

- [x] Unit tests: bad signature, bad checksum, unknown contract version, path traversal in the archive, interrupted install
- [ ] Integration test installs the template adapter from a local registry and syncs it
- [x] Installed AUR package installs and updates an adapter from the official registry on the author's machine
      2026-11-28, install half done: `wingdrive-bin` 3.8.2 on the laptop (isolated instance) ran
      `wing adapters browse` (index 9) and `wing adapters install obsidian`. Declining the
      permission question aborts and `--yes` installs 1.1.2 with a lockfile entry (commit 2a80a59,
      SHA-256, accepted permissions). A source made with it synced a two-note vault (2 items). The
      update half waits for a newer published version.
      Update half done the same day: after index 13 published Obsidian 1.1.3, `wing adapters outdated`
      listed "installed 1.1.2, update to 1.1.3", `wing adapters update obsidian --yes` printed "Updated
      obsidian from 1.1.2 to 1.1.3", the lockfile records 1.1.3 at 910dc81 and sequence 13, and the
      existing vault source synced again ("0 upserted, 0 deleted").
