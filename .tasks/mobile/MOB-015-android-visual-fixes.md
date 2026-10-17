---
id: MOB-015
title: "Android visual fixes: header buttons and system bar overlap"
status: Done
assignee: bernardopg
parent: MOB-000
priority: High
milestone: M5
sprint: S03
tags: [mobile, android, interface]
last_updated: 2026-10-17
---

## Description

On Android the round header buttons rendered as dark squares: the fallback wrapped `expo-blur` (which
does not clip to `borderRadius` on Android) around an inset square tint layer. The Jobs and Search
modals used a fixed 18 pt top padding meant for iOS sheets, so on Android their headers sat under the
status bar. With edge-to-edge on, Explorer lists and the Jobs list ran under the navigation bar.

## Acceptance Criteria

- [x] `GlassButton` fallback is one rounded, clipped view with a ripple; used by Overview, Explorer,
      Jobs, Search and the search toolbar
- [x] Jobs and Search headers pad by the top inset on Android
- [x] Explorer list, grid and the Jobs list pad by the bottom inset
- [x] Checked on the Moto G7: Overview, Jobs and Explorer headers
- [x] Overview no longer crashes intermittently at launch (2 of 3 launches before the fix)

## Evidence

- Root cause of the launch crash: `updateBatchResources` in `packages/ts-client` seeded an empty cache
  with the file-listing shape for every resource type. A device batch from sync arrived before the
  `devices.list` response, the cache became `{files, total_count, has_more}` and `DevicePanel` called
  `.map` on it. Now only file queries get the seed; test in `useNormalizedQuery.test.tsx`. The desktop
  shares this hook.
- 3 cold launches after the fix: 0 errors.
