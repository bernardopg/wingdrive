---
id: FORK-004
title: Publish WingDrive Linux releases and AUR package
status: In Progress
assignee: bernardopg
parent: FORK-002
priority: High
milestone: M1
sprint: S01
tags: [fork, release, linux, aur, packaging]
last_updated: 2026-10-01
---

## Description

Publish verified Linux desktop bundles as WingDrive and maintain wingdrive-bin on AUR.
Release tags must match every product manifest. Publishing follows artifact verification.

## Acceptance Criteria

- [ ] AppImage and deb include the daemon and start in an isolated production session
- [ ] GitHub Actions checks types, native tests, formatting, lint, versions, and bundle startup
- [ ] Published release includes SHA256SUMS and the license
- [ ] AUR metadata uses the published AppImage with verified checksums
- [ ] wingdrive-bin is published and its package build passes on Arch Linux
- [ ] Subsequent tagged releases update AUR idempotently

## Evidence

Pending completion of the production bundle and publication.
