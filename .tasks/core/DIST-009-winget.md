---
id: DIST-009
title: "winget: manifest in microsoft/winget-pkgs and updates from the release workflow"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: Medium
milestone: M5
sprint: S08
tags: [windows, winget, distribution, ci]
related_tasks: [REL-002, TAURI-016]
last_updated: 2026-11-28
---

## Description

winget is free, needs no account beyond GitHub, and accepts the NSIS installer, which installs
silently with `/S`. Unsigned installers are accepted but go through Microsoft's malware scan and
SmartScreen still warns; REL-002 improves both.

## Implementation Steps

- [ ] Pick the package ID `WingDrive.WingDrive` and run `wingetcreate new` with the stable `-x64-setup.exe` URL
- [ ] Sign the CLA on the first pull request and answer the validation
- [ ] Add a release job on `windows-latest` that runs `wingetcreate update WingDrive.WingDrive --version <v> --urls <url> --submit` for stable tags, with a classic PAT in `WINGET_TOKEN`
- [ ] Document the job and the secret in `docs/core/releases.mdx`

## Acceptance Criteria

- [ ] `winget install WingDrive.WingDrive` installs the latest stable release
- [ ] A stable tag opens the winget-pkgs pull request without manual steps
