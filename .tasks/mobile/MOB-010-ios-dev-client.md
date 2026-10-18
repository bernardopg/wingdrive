---
id: MOB-010
title: "iOS dev client on macOS or EAS"
status: To Do
assignee: bernardopg
parent: MOB-000
priority: Low
milestone: M5
tags: [mobile, ios]
last_updated: 2026-10-17
---

## Description

iOS builds need Xcode, so they cannot run on the Linux workstation. Pick a Mac or EAS Build. EAS
needs an `eas-build-pre-install` hook that installs `rustup` and `aarch64-apple-ios`.

## Acceptance Criteria

- [ ] iOS dev client installed on a simulator or iPhone and loading JS from Metro
- [ ] Steps recorded in `docs/core/mobile-development.mdx`
