---
id: DIST-012
title: "Snap Store: strictly confined snap"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: Low
milestone: M5
sprint: S12
tags: [linux, snap, distribution]
related_tasks: [DIST-008]
last_updated: 2026-11-28
---

## Description

Ubuntu users find apps in the Snap Store. Store reviewers refuse classic confinement and
`removable-media` auto-connect for file managers, so the snap runs strictly confined with `home`
(auto-connected on classic distributions) and a manually connected `removable-media`. Lower
priority than Flathub, which most other distributions use.

## Implementation Steps

- [ ] Register the snap name `wingdrive`
- [ ] Write `snap/snapcraft.yaml` that repackages the `.deb` (core24, gnome extension), plugs `home`, `removable-media`, `network`, `network-bind`
- [ ] Detect missing `removable-media` with `snapctl is-connected` and tell the user to run `snap connect wingdrive:removable-media`
- [ ] Publish to the edge channel from the release workflow with `SNAPCRAFT_STORE_CREDENTIALS`, stable by hand

## Acceptance Criteria

- [ ] `snap install wingdrive` installs the app and the daemon starts
