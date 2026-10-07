---
id: DESK-009
title: Network locations and devices through GIO/gvfs
status: To Do
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

- [ ] List GIO mounts and mountable volumes (gvfs) including MTP and network mounts
- [ ] Mount and unmount through `gio mount`, with password prompts in the UI
- [ ] "Connect to Server" dialog for `smb://`, `sftp://`, `ftp://`, `dav://` URIs
- [ ] Browse mounted gvfs paths under `/run/user/<uid>/gvfs` ephemerally

## Acceptance Criteria

- [ ] A phone in MTP mode appears in the sidebar and can be browsed
- [ ] `smb://host/share` connects, browses and unmounts
