---
id: DESK-095
title: Status bar hides free space for folders opened from outside the app
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S05
tags: [desktop, explorer, bug]
last_updated: 2026-11-12
---

## Description

The Explorer status bar (DESK-027) finds the device of the current folder by comparing the folder's
`device_slug` with each device's slug, then picks that device's volume for the free space. Folders
opened from outside the app (`xdg-open`, a second launch, "Show in folder") use Explorer URLs built by
`explorerUrlForDirectory`, which name this device `local`. No device has the slug `local`, so the free
space part stays hidden for every folder opened that way, while the same folder opened from the
sidebar shows it. DESK-050 fixed the same slug mismatch for drops. Found while checking DESK-027 in the
web UI on 2026-11-12.

## Steps to reproduce

1. Isolated `wing-server`, open `/explorer?path={"Physical":{"device_slug":"local","path":"<folder>"}}`.
2. The status bar reads `5 folders · 6 files` with no free space.
3. Open the same folder with the device's real slug (`archlinux`): it reads `5 folders · 6 files` and
   `362 GB free`.

## Acceptance Criteria

- [x] A folder opened with the `local` slug shows the free space of its volume
- [x] `statusBar.test.ts` covers the slug lookup
- [x] Checked in the web UI

## Fix

`deviceIdForSlug` in `statusBar.ts` resolves a slug to a device id and treats `local` as the device
with `is_current`, the rule `sameDevice` uses for drops. `ExplorerStatusBar` uses it. "matches the
slug, and `local` names the current device" in `statusBar.test.ts` covers it (8 tests pass).

## Evidence

Isolated `wing-server` (debug build, own data dir and instance, port 18093), web UI in headless Chrome,
2026-11-12: the fixture folder opened with the `local` slug read `5 folders · 6 files` and `361 GB
free` after the fix, the same as with the real slug.
