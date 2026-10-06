---
id: FORK-004
title: Publish WingDrive Linux releases and AUR package
status: Done
assignee: bernardopg
parent: FORK-002
priority: High
milestone: M1
sprint: S01
tags: [fork, release, linux, aur, packaging]
last_updated: 2026-10-06
---

## Description

Publish verified Linux desktop bundles as WingDrive and maintain wingdrive-bin on AUR.
Release tags must match every product manifest. Publishing follows artifact verification.

## Acceptance Criteria

- [x] AppImage and deb include the daemon and start in an isolated production session
- [x] GitHub Actions checks types, native tests, formatting, lint, versions, and bundle startup
- [x] Published release includes SHA256SUMS and the license
- [x] AUR metadata uses the published AppImage with verified checksums
- [x] wingdrive-bin is published and its package build passes on Arch Linux
- [x] Subsequent tagged releases update AUR idempotently

## Evidence

- Tag `v2.0.0-alpha.6` (b17f989): WingDrive Release run 12 succeeded on 2026-10-04.
- Release assets: `WingDrive-2.0.0-alpha.6-x86_64.AppImage`, `WingDrive-2.0.0-alpha.6-amd64.deb`, `LICENSE`, `PKGBUILD` and `SHA256SUMS`.
- AUR `wingdrive-bin` 2.0.0alpha6-1 is published. Its AppImage sha256 `8ddd18ce...6577` matches `SHA256SUMS`.
- `release.yml` builds the package with makepkg in an Arch container before pushing to AUR.

Open: idempotent AUR update is automated but unproven. Close this task when the next tag (`v2.0.0-alpha.7`) updates `wingdrive-bin` without manual steps.

- Tag `v2.0.0-alpha.7` (5a3e36d): WingDrive Release run 37408850907 succeeded on 2026-10-06.
  The AUR job built and smoke-tested the package in Arch and pushed `wingdrive-bin` commit
  `ca47c4e` (`pkgver=2.0.0alpha7`, `license=('Apache-2.0' 'GPL-3.0-only')`) with no manual
  step. The AppImage sha256 `4569f2da...be49` matches `SHA256SUMS`. Closed.
