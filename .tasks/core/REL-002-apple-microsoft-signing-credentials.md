---
id: REL-002
title: Apple and Microsoft signing credentials for release signing
status: To Do
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M3
sprint: S08
tags: [release, signing, windows, macos, blocked]
related_tasks: [REL-001, TAURI-016]
last_updated: 2026-11-22
---

## Description

REL-001 requires WingDrive-owned signing credentials. Apple notarization/signing and Microsoft Authenticode certificates are not available in the current development environment. Record as S08 blocker.

## Acceptance Criteria

- [ ] Apple Developer ID / notarization credentials configured for macOS release signing.
- [ ] Microsoft Authenticode certificate configured for Windows release signing.
- [ ] CI secrets documented and release pipeline validates signed artifacts.
