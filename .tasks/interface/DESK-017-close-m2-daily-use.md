---
id: DESK-017
title: "Close M2: AUR runtime evidence and a full working day"
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S04
tags: [desktop, linux, release, verification]
last_updated: 2026-10-16
---

## Description

Every DESK child task is Done in code, but the epic closes only on the
published AUR package and on a real working day without falling back to
another file manager. Several recent changes were verified by tests only:
directory sizes (DESK-014), the translated tray, ephemeral rename events
(#208) and vanished volumes going offline (#210).

## Implementation Steps

1. Cut a release with the merged work and install `wingdrive-bin` from the AUR.
2. Run the DESK checklist on the installed package: `xdg-open` on a folder,
   "Show in folder" from a browser download, tray and autostart, Open With,
   terminal here, new file and symlink, permissions, archives, gvfs (SFTP/SMB
   or MTP), folder sizes, Portuguese UI and tray, open a folder in a new window
   (DESK-015), renaming a folder in an
   unindexed directory, unplugging a USB drive.
3. Use WingDrive as the only file manager for a full working day and record
   every fallback or bug as a task.
4. Record the evidence in DESK-000 and close the epic and the M2 milestone.

## Progress

- 2026-10-15: release 2.0.0-alpha.16 prepared with #205 to #217.

## Acceptance Criteria

- [ ] DESK checklist passes on the installed AUR package
- [ ] A full working day with no fallback to another file manager
- [ ] Bugs found are filed as tasks; DESK-000 is Done
