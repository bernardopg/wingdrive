---
id: DESK-002
title: org.freedesktop.FileManager1 D-Bus service
status: Done
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [desktop, linux, dbus]
last_updated: 2026-10-07
---

## Description

Browsers, editors and portals call `org.freedesktop.FileManager1.ShowItems` to reveal a file. WingDrive only calls that interface. It must own the name while running and implement `ShowFolders`, `ShowItems` and `ShowItemProperties`.

## Implementation Steps

- [x] Own `org.freedesktop.FileManager1` on the session bus with `zbus` when the name is free
- [x] Implement `ShowFolders`, `ShowItems` (select) and `ShowItemProperties` (open Inspector)
- [x] Reveal from WingDrive itself keeps working when it owns the name
- [x] D-Bus activation file so the call starts WingDrive when it is not running

## Acceptance Criteria

- [x] `gdbus call ... ShowItems ['file:///etc/hosts'] ''` opens /etc with hosts selected
- [x] "Show in folder" from Firefox opens WingDrive

## Evidence

- `apps/tauri/src-tauri/src/filemanager1.rs` serves `ShowFolders`, `ShowItems` and `ShowItemProperties` with `zbus` and
  requests the name with `ReplaceExisting | AllowReplacement`.
- Private `dbus-run-session` with Xvfb: WingDrive logged `reply=PrimaryOwner`; `gdbus call ... ShowItems
  ['file:///etc/hosts'] ''` opened an `etc` tab, scrolled to `hosts` and selected it.
- On the author's session `dolphin --daemon` already held the name without allowing replacement, so WingDrive was
  queued (`reply=InQueue`) and takes over when Dolphin exits. With WingDrive started at login it owns the name first.
- The AUR package installs `com.wingdrive.FileManager1.service` so the call starts WingDrive hidden when it is not running.
- Grid and list views scroll virtualized rows to the revealed file.
