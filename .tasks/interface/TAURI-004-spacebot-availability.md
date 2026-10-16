---
id: TAURI-004
title: Make Spacebot Availability Explicit
status: In Progress
assignee: unassigned
parent: TAURI-000
priority: High
milestone: M5
tags: [tauri, spacebot, integration, upstream-backlog]
last_updated: 2026-10-16
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

## WingDrive triage (2026-10-16)

Inherited from upstream Spacedrive. It is not on WingDrive's desktop file-manager path, so it
is parked in the Later milestone M5 (`.tasks/ROADMAP.md`). Re-decide whether WingDrive wants it
before pulling it into a sprint.
