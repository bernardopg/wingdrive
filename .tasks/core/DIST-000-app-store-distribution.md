---
id: DIST-000
title: "App store distribution: Google Play, F-Droid, Flathub, winget and the other stores"
status: To Do
assignee: bernardopg
priority: High
milestone: M5
sprint: S08
tags: [epic, release, distribution, android, linux, windows, macos]
related_tasks: [REL-001, REL-002, MOB-009, INTG-003]
last_updated: 2026-11-28
---

## Description

WingDrive ships from GitHub Releases, AUR and GHCR. This epic publishes it in the stores that
reach users who never visit GitHub. The roadmap, the order of stores, costs and the blockers found
in the code are in `docs/design/app-store-distribution.md`.

## Children

- Android: DIST-001, DIST-002, DIST-003 (Google Play), MOB-018, MOB-019, MOB-020 (blockers and
  metadata), DIST-004 (Obtainium), DIST-005 (developer verification), DIST-006 (F-Droid),
  DIST-007 (IzzyOnDroid)
- Desktop: DIST-008 (Flathub), DIST-009 (winget), DIST-010 (Microsoft Store), DIST-011 (Homebrew),
  DIST-012 (Snap Store)

## Acceptance Criteria

- [ ] WingDrive is installable from Google Play, F-Droid, Flathub and winget
- [ ] Every store listing links from the website download page and the README
- [ ] Each store updates from the release workflow or from the store's own bot, with no manual upload
