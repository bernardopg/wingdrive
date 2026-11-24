---
id: ADAPT-018
title: "Update notifications for adapters and WingDrive"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, updates, notifications, interface]
depends_on: [ADAPT-004]
related_tasks: [ADAPT-005, ADAPT-001]
last_updated: 2026-11-24
---

## Description

Users must know when an update is available, for WingDrive itself and for each installed adapter.
Today only the AppImage and the Windows installers check for app updates; packages managed by
pacman, apt or Flatpak say nothing until the user upgrades by hand, and adapters have no update
check at all.

## Implementation Steps

- [ ] Daemon job that refreshes the signed adapter index on start and every 6 hours, compares it with the lockfile and emits an `AdapterUpdatesAvailable` event; security updates and revocations flagged as such
- [ ] App update check for every install type: the existing updater for AppImage and Windows, and a read of the latest GitHub release for AUR, `.deb`, Flatpak and macOS that tells the user which command or store updates them
- [ ] Indicators: a badge on Sources > Adapters and on Settings > General > Updates, a tray menu entry, and one desktop notification per new version (not repeated)
- [ ] Settings: check automatically (on by default), notify for adapters, install adapter updates automatically (off by default, never for updates that ask for more permissions)
- [ ] A revoked adapter version is disabled at once with a notification that explains why
- [ ] `wing update` and `wing adapters outdated` report the same state

## Acceptance Criteria

- [ ] Interface tests for the badges and the notification preferences
- [ ] Real check on the author's machine: an older adapter pinned in a local registry shows an update, installs it, and the badge clears; the AUR install shows a newer WingDrive release
- [ ] No notification repeats for the same version after a restart
