---
id: TAURI-004
title: Make Spacebot Availability Explicit
status: In Progress
assignee: unassigned
parent: TAURI-000
priority: High
tags: [tauri, spacebot, integration]
last_updated: 2026-08-26
---

## Description

The tracked fallback `@spacebot/api-client` returns empty data and successful no-op mutations when the private Spacebot repository is absent. The desktop UI must never report a successful chat, task update, cancellation, or deletion in that mode.

## Acceptance Criteria

- [x] Build exposes whether the real Spacebot client is available
- [x] Spacebot entry point is hidden or shows an unavailable screen without it
- [x] Fallback mutations reject with an actionable error
- [x] Memories, Dream, and Schedule stay hidden until implemented
- [ ] Real-runtime contract test covers conversations and tasks

## Validation

- The unavailable build renders an explicit screen and does not mount Spacebot queries or mutations.
- Every fallback mutation rejects with the same actionable error; the focused Bun test covers all seven mutation methods.
- Placeholder routes for Memories, Autonomy, and Schedule are no longer registered.
- Typecheck and the production Vite build pass without the private repository.
- The private `spacebot/packages/api-client` repository is absent on this system, so the real-runtime contract remains pending.
- All criteria implementable without the private repository are complete (commit 6ccd013bb). The remaining contract test is blocked on access to `spacebot/packages/api-client`; reassign or unblock to finish.
