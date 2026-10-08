---
id: DESK-009
title: Network locations and devices through GIO/gvfs
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S03
tags: [desktop, linux, gvfs, volumes]
last_updated: 2026-10-07
---

## Description

Phones (MTP), SMB shares and SFTP servers are reached through gvfs on Linux desktops. WingDrive only sees mounted volumes.

## Implementation Steps

- [x] List GIO mounts and mountable volumes (gvfs) including MTP and network mounts
- [x] Mount and unmount through `gio mount`, with password prompts in the UI
- [x] "Connect to Server" dialog for `smb://`, `sftp://`, `ftp://`, `dav://` URIs
- [x] Browse mounted gvfs paths under `/run/user/<uid>/gvfs` ephemerally

## Acceptance Criteria

- [x] A phone in MTP mode appears in the sidebar and can be browsed (parser covered by a unit test; no MTP device or `gvfs-mtp` on the test machine)
- [x] A network address connects, browses and unmounts (verified with SFTP; SMB uses the same path but `gvfs-smb` is not installed here)

## Evidence

- `apps/tauri/src-tauri/src/gvfs.rs`: lists FUSE folders under `$XDG_RUNTIME_DIR/gvfs` with readable names, parses
  `gio mount -li` for mountable volumes, mounts with `gio mount` (password on stdin, 90 s timeout, stdin closed so an
  unanswerable prompt fails instead of hanging) and unmounts only folders under the gvfs root. 3 unit tests.
- UI: network mounts and mountable devices under Volumes, polled every 5 s / 10 s, with Unmount on hover and a
  Connect to Server dialog. AUR optdepends list gvfs, gvfs-mtp, gvfs-smb, gvfs-gphoto2.
- Runtime in a private D-Bus session with its own gvfs: `sftp://homesystem/` mounted, the explorer opened the remote
  root, the sidebar showed `homesystem (SFTP)`, and Unmount removed it. A failing address showed gio's error in the
  dialog ("Conexão falhou") instead of hanging.
- Unmounting while browsing the mount navigates the tab away from the stale folder.
