---
id: CLI-002
title: wing update finds no binaries in WingDrive releases
status: Done
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

- [x] Decide the behavior per install kind: package manager (AUR, `.deb`), Windows installer,
      AppImage, source build. The path of the running binary decides: `/opt/wingdrive` is the
      AUR (pacman names the package), `/usr` the `.deb`, `$APPIMAGE` the AppImage, Program Files
      the MSI, any other Windows path the NSIS installer, a `target/` folder a source build
- [x] For package-managed installs, report the new version and print the package manager command
      instead of downloading anything
- [x] Remove the in-place replacement: `wing update` downloads nothing, and `--force` is gone
- [x] Update `docs/core/cli.mdx`: "Checking for Updates" replaces "Auto-Update System"

## Acceptance Criteria

- [x] `wing update` on the AUR package reports the latest release and the `yay` command, and
      changes no file (`instructions` for `InstallKind::Aur`; `pacman -Qqo` on the author's
      machine names `wingdrive-bin`)
- [x] `wing update` never ends with "Could not find WingDrive CLI for platform" against
      `bernardopg/wingdrive`: run on 2026-11-05 on the `stable` and `nightly` channels, it
      reports 3.0.0 as the latest release
- [x] A test covers the asset selection against the asset names of a real release (`v3.0.0`):
      `points_each_install_kind_at_a_real_v3_asset`, plus four more tests in `update::tests`

## Notes

Found on 2026-11-05 while checking the documentation after the `v3.0.0` release, and fixed the
same day. A pre-release sends AUR users to `wingdrive-nightly`, the package that carries them.
