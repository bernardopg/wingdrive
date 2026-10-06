---
id: BRAND-002
title: Wingbot WingUI Wingdrop and Wings branding
status: Done
assignee: bernardopg
parent: UI-000
priority: High
milestone: M1
sprint: S02
tags: [branding, interface, compatibility]
last_updated: 2026-10-06
---

## Description

The user selected brand and UI renaming: Wingbot, WingUI, Wingdrop and Wings. Preserve existing API names, persistence, database schemas and external integration compatibility. Do not replace technical terms such as whitespace, namespace, workspace or color space, nor legally required upstream attribution. No changes to repositories outside this workspace.

## Acceptance Criteria

- [x] User-facing product names and documentation use Wingbot, WingUI, Wingdrop and Wings
- [x] Owned frontend entry points and imports are renamed consistently where practical
- [x] Existing RPC, persisted data and legacy URLs remain compatible
- [x] External package/service identifiers are preserved or adapted explicitly, not blindly replaced
- [x] Automated checks guard against old visible branding and verify compatibility
- [x] Typecheck, tests and build pass after the rename

## Validation

Implemented on 2026-10-06.

| Old | New | Renamed | Kept for compatibility |
|---|---|---|---|
| Spacebot | Wingbot | `packages/interface/src/Wingbot/` (`WingbotProvider`, `WingbotLayout`, `useWingbotEventSource`), `@wingdrive/interface/Wingbot` export, `/wingbot` routes, Tauri window `wingbot`, query keys, UI text | `@spacebot/api-client` (external runtime package), `AppConfig.spacebot` and `spacebot_*` RPC fields, `SpacebotConfigOutput`; `/spacebot/*` URLs and `spacebot` window labels redirect to `/wingbot` |
| Spacedrop | Wingdrop | `Wingdrop` component and window, `/wingdrop` route, menu item, `wing network wingdrop` | `network.spacedrop.send` RPC and `SpacedropSend*` types; `wing network spacedrop` alias; `spacedrop` window label |
| SpaceUI | WingUI | Package READMEs and token comments | MIT attribution in NOTICE.md and CONTRIBUTING; `../spaceui` local checkout paths in the Vite configs |
| Spaces | Wings | "Create Wing", "Wing Name", "Select Wing", "Remove from Wing", `wing wings` CLI output and docs | `spaces.*` RPC, `space` tables and types; `wing spaces` alias |

Unchanged on purpose: the Space key, DigitalOcean Spaces, Storage Spaces, and the external runtime's design docs
(`docs/core/design/spacebot-*.md` now open with a naming note).

Checks:
- `scripts/check-wingdrive-independence.sh` fails on `Spacebot`, `Spacedrop` or `SpaceUI` in frontend and CLI
  string literals (verified by injecting one).
- `legacyWingbotPath` unit tests; the Spacebot fallback stub test still passes.
- `bun run typecheck`, `cargo clippy --workspace -D warnings` and `cargo fmt --check` pass.
- `wing network wingdrop --help` and the `spacedrop` alias both resolve.
