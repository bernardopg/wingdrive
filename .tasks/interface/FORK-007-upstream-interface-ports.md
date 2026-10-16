---
id: FORK-007
title: Port upstream interface fixes (local-only open, modifier drag, paired devices)
status: To Do
assignee: bernardopg
parent: FORK-002
priority: High
milestone: M2
sprint: S03
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

- [ ] Open With and Reveal are hidden for files on another device
- [ ] Ctrl-drag copies and Shift-drag moves
- [ ] Paired devices show in the overview without a connection event
- [ ] `bun run typecheck` and the interface tests pass
