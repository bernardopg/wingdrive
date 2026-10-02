---
id: TAURI-010
title: System Clipboard Integration for Files
status: Done
assignee: bernardopg
parent: TAURI-000
priority: Medium
sprint: S01
milestone: M1
tags: [tauri, clipboard, explorer]
last_updated: 2026-10-02
---

## Description

Copy and paste inside WingDrive uses an internal clipboard only. Files copied in WingDrive cannot be pasted in another file manager and the reverse.

## Acceptance Criteria

- [x] Copy in WingDrive writes file URIs to the system clipboard on Linux
- [x] Paste in WingDrive accepts file URIs copied from another file manager
- [x] Cut/paste keeps move semantics
- [x] No `console.log` in the clipboard path

## Verification (2026-10-02)

Linux GTK clipboard parser tests cover GNOME and KDE move markers, local URI validation and invalid input. Native IPC exports and imports file URI lists; cut state is cleared after successful paste only.
