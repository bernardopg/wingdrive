---
id: CLI-002
title: wing update finds no binaries in WingDrive releases
status: To Do
assignee: bernardopg
parent: CLI-000
priority: Medium
milestone: M3
tags: [cli, release, updater]
related_tasks: [REL-001]
last_updated: 2026-11-05
---

## Description

`wing update` (`apps/cli/src/domains/update/mod.rs`) reads the latest GitHub Release and looks for
assets whose names contain `wingdrive-cli` and the platform (`linux-x86_64`, `windows-x86_64`,
...). The Release workflow never publishes such assets: it ships the AppImage, the `.deb`, the
Windows installers and the Android bundles, with `wing` inside them. So `wing update` always stops
with "Could not find WingDrive CLI for platform". The command was inherited from the upstream.

Replacing the binaries in place would also be wrong for most installs: the AUR package and the
`.deb` own `/usr/bin/wing`, and the Windows installers own their folder.

## Implementation Steps

- [ ] Decide the behavior per install kind: package manager (AUR, `.deb`), Windows installer,
      AppImage, source build
- [ ] For package-managed installs, report the new version and print the package manager command
      instead of downloading anything
- [ ] Either publish the assets the command expects, signed (REL-001), or remove the in-place
      replacement
- [ ] Update `docs/core/cli.mdx`, section "Release Repository Setup"

## Acceptance Criteria

- [ ] `wing update` on the AUR package reports the latest release and the `yay` or `pacman`
      command, and changes no file
- [ ] `wing update` never ends with "Could not find WingDrive CLI for platform" against
      `bernardopg/wingdrive`
- [ ] A test covers the asset selection against the asset names of a real release (`v3.0.0`)

## Notes

Found on 2026-11-05 while checking the documentation after the `v3.0.0` release.
