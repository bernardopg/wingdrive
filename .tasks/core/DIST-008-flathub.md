---
id: DIST-008
title: "Flathub: app ID on wingdrive.org, metainfo and the flathub submission"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: High
milestone: M5
sprint: S09
tags: [linux, flatpak, flathub, distribution, website]
related_tasks: [REL-003, INTG-001]
last_updated: 2026-11-28
---

## Description

The release already builds a Flatpak bundle (`packaging/flatpak/com.wingdrive.desktop.yml`) by
repackaging the `.deb`. Flathub needs three changes. The app ID must sit on a domain the project
controls, and `wingdrive.com` is a parked page, so `com.wingdrive.desktop` would be refused; use
`org.wingdrive.WingDrive`. The metainfo needs screenshots and releases. `--filesystem=host` needs a
linter exception with a file-manager justification. The Tauri identifier and data directories do
not change.

## Implementation Steps

- [ ] Rename the Flatpak app ID to `org.wingdrive.WingDrive` in the manifest, metainfo `<id>`, desktop file and icon names, and in `flatpak-bundle.yml`
- [ ] Metainfo: `<screenshots>` with hosted images, `<releases>` generated from CHANGELOG, `<branding>` colors, `<launchable>`, OARS answers; pass `flatpak run --command=flatpak-builder-lint org.flatpak.Builder appstream`
- [ ] Manifest for Flathub: the `.deb` from the GitHub Release by URL and sha256, with `x-checker-data` so the Flathub bot opens update PRs
- [ ] Show "update with flatpak update" instead of the pacman hint when `FLATPAK_ID` is set (`apps/tauri/src-tauri/src/updates.rs`)
- [ ] Open the PR to github.com/flathub/flathub (branch `new-pr`) with the `--filesystem=host` justification
- [ ] After acceptance, publish the token at `website/public/.well-known/org.flathub.VerifiedApps.txt`

## Acceptance Criteria

- [ ] `flatpak install flathub org.wingdrive.WingDrive` installs and starts the app and daemon
- [ ] The app shows as verified on flathub.org
- [ ] A new release reaches Flathub through the bot PR
