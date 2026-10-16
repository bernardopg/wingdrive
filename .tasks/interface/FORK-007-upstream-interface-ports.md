---
id: FORK-007
title: Port upstream interface fixes (local-only open, modifier drag, paired devices)
status: In Progress
assignee: bernardopg
parent: FORK-002
priority: High
milestone: M2
sprint: S04
tags: [fork, upstream, port, interface]
last_updated: 2026-10-16
---

## Description

The 2026-10-16 survey of upstream mobile, Tauri and interface commits
(`docs/design/UPSTREAM_INTERFACE_SURVEY.md`)
found three small interface fixes worth porting for the Linux file manager.
None cherry-picks cleanly because of the `@sd`/`sd_path` renames, so each is
re-applied by hand and credits upstream in the commit body.

## Implementation Steps

1. `e752cb460`: show Open With and Reveal only for files on this device
   (`useFileContextMenu.ts`). A paired device's path must not reach the local
   opener.
2. `313a1da12` (hook only): add `hooks/useModifierKeys.ts`; in
   `DndProvider.tsx`, Ctrl forces copy and Shift forces move, with the
   same-volume guess as the default.
3. `9d948817e`: pass `show_paired` in the overview `DevicePanel.tsx`.

## Acceptance Criteria

- [x] Open With and Reveal are hidden for files on another device
- [x] Ctrl-drag copies and Shift-drag moves
- [ ] Paired devices show in the overview without a connection event
- [x] `bun run typecheck` and the interface tests pass

## Progress

- 2026-10-16, step 1: `useFileContextMenu.ts` keeps only `is_local` paths.
  Open With and Always Open With need every target local, Reveal needs the
  clicked file local, and Share needs every target local. The inspector
  share button in `FileInspector.tsx` has the same guard. Remote open and
  streaming (`files.local_path`, `files.stream_url`, `useOriginalUrl`) are
  not ported, because the core operations do not exist in WingDrive.
- 2026-10-16, step 2: added `hooks/useModifierKeys.ts` (tracker,
  `modifierKeysNow`, `dropOperation`) and `useModifierKeys.test.ts`.
  `DndProvider.tsx` mounts the tracker. Ctrl forces copy and Shift forces
  move. With neither key or both keys held, the same-volume guess is used.
  The unused `useModifierKeys()` selector was left out.
- 2026-10-16, step 3: the overview `DevicePanel.tsx` passes
  `show_paired: true`.
- Checks: `bun run typecheck` passes. `bun test` in `packages/interface`
  passes (77 tests). The paired-device criterion still needs a runtime
  check with a paired peer, so it stays open and the status stays In
  Progress.

