---
id: MOB-001
title: "Launch crash: duplicate @tanstack/react-query in the bundle"
status: Done
assignee: bernardopg
parent: MOB-000
priority: High
milestone: M5
sprint: S03
tags: [mobile, bug, metro]
last_updated: 2026-10-17
---

## Description

The app dies on launch with `No QueryClient set, use QueryClientProvider to set one` in
`OverviewScreen`. Bun installs one copy of `@tanstack/react-query` for `apps/mobile` and another for
`packages/ts-client`. `QueryClientProvider` comes from the first, `useJobs` and `useNormalizedQuery`
from the second, so the React contexts differ.

## Acceptance Criteria

- [x] Metro resolves `@tanstack/react-query`, `react` and `react-native` to one copy each
- [x] The app opens to the Overview screen on the emulator and on the Moto G7
- [x] No `FATAL EXCEPTION` in logcat during launch

## Evidence

- `metro.config.js` `resolveRequest` resolves every package in `apps/mobile/package.json` dependencies
  from the app, whoever imports it (covers ts-client's react-query and expo-router's screens 4.29).
- 2026-10-17, emulator `wingdrive-api36` debug dev client: app opens to "My Library" Overview, zero
  `FATAL`/`No QueryClient` lines in logcat.
- 2026-10-17 Moto G7 debug build with the new core: app opens, `Mobile core initialized successfully`, no `FATAL`.
