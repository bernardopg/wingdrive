---
id: DESK-050
title: File drops default to Copy on the same device in the desktop build
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S05
tags: [desktop, explorer, dnd, bug]
last_updated: 2026-11-04
---

## Description

`DndProvider.dropFilesInto` guesses Move when the source and destination share a device slug, as Linux file
managers do, and Ctrl or Shift override the guess. In the desktop debug build under Xvfb on 2026-11-04 the
copy/move dialog opened with Copy for every drop, including a file dropped on a folder in the same pane,
with no modifier held (pointer events reported `ctrlKey` false). Found while checking DESK-016.

## Acceptance Criteria

- [x] A drop within one device opens the dialog on Move, and Ctrl switches it to Copy
- [x] Root cause recorded (slug mismatch, stale modifier state, or another path)
- [x] Verified on the Linux desktop build

## Root cause

Listings copy the `device_slug` of the URL they were opened with into every file: Explorer URLs built by
`explorerUrlForDirectory` say `local`, while sidebar locations and device views carry the real slug
(`archlinux`). `dropFilesInto` guessed Move only when both slugs were equal strings, so any drop between
views opened from different places guessed Copy. DESK-016 made it worse by rewriting only the destination's
`local` to the real slug, which made every drop within one pane differ too.

## Evidence

- `components/dropTarget.ts`: `sameDevice` treats `local` and this device's slug as one device;
  `guessDropOperation` and `alreadyIn` use it; `DndProvider` drops the one-sided rewrite and its duplicate
  helper. `dropTarget.test.ts` covers both slug orders, other devices, clouds and files already in place.
- Desktop debug build, 2026-11-04 (Xvfb, private D-Bus, isolated daemon, xdotool drags): a file dropped on a
  folder in the same pane opened the dialog on Move, and on Copy with Ctrl held. A drop from a pane opened by
  URL (`local`) onto a pane opened from the sidebar location (`archlinux`) opened on Move, also after closing
  and reopening the split; confirming it moved `notes2.md` from `docs` to `photos` on disk.
- One run earlier in the same check showed no dialog for the cross-pane drop; it did not recur in seven later
  attempts across relaunches, F3 toggles and sidebar navigation.
