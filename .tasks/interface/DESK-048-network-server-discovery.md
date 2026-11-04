---
id: DESK-048
title: Discover file servers on the network
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S04
tags: [desktop, linux, network, gvfs]
last_updated: 2026-11-03
---

## Description

SFTP, SMB, FTP and WebDAV already mount through gvfs (DESK-009), but the address had to be typed.
The Network section now lists servers it finds: services announced over mDNS/DNS-SD (`_sftp-ssh`,
`_ssh`, `_smb`, `_ftp`, `_webdav`, `_webdavs`, `_nfs`) read from `avahi-browse`, plus the hosts in
`~/.ssh/config`, which covers machines such as `homesystem` that do not announce themselves.

## Acceptance Criteria

- [x] Servers on the network and SSH hosts appear under Network
- [x] A click mounts the server and opens it; if it needs a password or user, the connect dialog opens on its address with the error
- [x] Verified in the desktop app against a real server

## Evidence

- `apps/tauri/src-tauri/src/network_discovery.rs`: `discover_network_servers` command, `avahi-browse` capped at 5 s (on the author's network it never exits on its own; the partial output had 50 resolved services). 3 unit tests (mDNS parsing, SSH config, escapes).
- UI: `NetworkSection.tsx`, list refreshed every 60 s with a refresh button; servers already mounted are hidden.
- Desktop debug build, 2026-11-03: `target/debug/WingDrive` under Xvfb with a private D-Bus session, isolated data dir, instance, XDG_DATA_HOME and XDG_RUNTIME_DIR, driven through WebKitWebDriver with real key events. The Network section listed the 10 hosts of `~/.ssh/config`. With two services published by
  `avahi-publish` the command also returned `WD Teste SMB (SMB) | smb://archlinux.local/` and
  `WD Teste DAV (WebDAV) | dav://archlinux.local:8080/dav`. Clicking `homesystem (SFTP)` mounted it through
  gvfs and opened its root (19 folders); the server moved from "On this network" to the mounted list, and
  Unmount moved it back. A failed mount (gvfs FUSE unavailable in a first run) opened the connect dialog on
  the address with the error.
- The author's network announces no SFTP, SMB, FTP, WebDAV or NFS service, so only SSH hosts show there.
  SMB needs `gvfs-smb`, not installed on the test machine.
