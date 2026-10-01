---
id: TAURI-010
title: System Clipboard Integration for Files
status: To Do
assignee: bernardopg
parent: TAURI-000
priority: Medium
sprint: S02
milestone: M1
tags: [tauri, clipboard, explorer]
last_updated: 2026-10-01
---

## Description

Copy and paste inside WingDrive uses an internal clipboard only. Files copied in WingDrive cannot be pasted in another file manager and the reverse.

## Acceptance Criteria

- [ ] Copy in WingDrive writes file URIs to the system clipboard on Linux
- [ ] Paste in WingDrive accepts file URIs copied from another file manager
- [ ] Cut/paste keeps move semantics
- [ ] No `console.log` in the clipboard path
