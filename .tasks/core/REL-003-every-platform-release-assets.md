---
id: REL-003
title: Release assets for every distribution, with a fixed release page and changelog format
status: Done
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M3
sprint: S05
tags: [release, ci, packaging, flatpak, macos, server, docs]
related_tasks: [REL-001, REL-002, TAURI-016]
last_updated: 2026-11-24
---

## Description

Until 3.6.1 a release shipped the AppImage, the `.deb`, the `PKGBUILD`, the Windows `.exe` and
`.msi` and the Android `.apk` and `.aab`, with three partial checksum files and GitHub's default
notes. Users of Flatpak, macOS and headless servers without Docker had nothing to download, and the
release page had no fixed shape.

This task adds the missing formats to `release.yml`, makes the publish job refuse an incomplete
release, writes the release body from `CHANGELOG.md`, and checks the changelog format in CI.

## Implementation Steps

1. `packaging/flatpak/` manifest and metainfo; `flatpak-bundle.yml` builds the bundle from the run's
   `.deb`, installs it and starts the daemon inside the sandbox.
2. `macos-bundle.yml`: Apple silicon `.app`, daemon re-signed with its own entitlements, ad hoc seal,
   `.dmg`, mounted and started in CI.
3. `scripts/release/package_server.py`: server and CLI archives for Linux (`.tar.gz`, `.tar.xz`,
   `.zip`) and the Windows CLI `.zip`, each extracted and started in CI.
4. `SRCINFO` next to `PKGBUILD`; one `SHA256SUMS` for every file.
5. `scripts/release/changelog.py` (format check in CI, `require` before a release build) and
   `scripts/release/notes.py` (release body, required asset list).
6. Document the asset list, naming rules, changelog standard and release page layout in
   `docs/core/releases.mdx` and `AGENTS.md`; show the new files on the website's download page.

## Acceptance Criteria

- [x] Flatpak bundle built and started locally from the 3.5.0 `.deb` (UI and daemon up in an isolated
  instance, 2026-11-24)
- [x] Server archives extract and run with an empty environment (all three formats, 2026-11-24)
- [x] `changelog.py check` passes on the whole file and runs in the "Repository Metadata" job
- [x] `notes.py` writes the body for v3.6.1 against the real GitHub data (73 pull requests grouped)
- [x] `actionlint` passes on every workflow
- [x] The publish job refuses a release that lacks a required asset (`notes.py` exits 1 and names it)

## Notes

- GitHub drops a leading dot from asset names, so `.SRCINFO` ships as `SRCINFO`.
- macOS is ad hoc signed until REL-002 brings a Developer ID; it is not in `latest.json`.
- Inside the Flatpak sandbox, volume detection sees fewer mounts than a native install.
