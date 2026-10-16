---
id: SETS-000
title: "Epic: Settings Interface"
status: In Progress
assignee: jamiepine
parent: UI-000
priority: Medium
tags: [epic, settings, interface, upstream-backlog]
whitepaper: N/A
last_updated: 2026-10-16
milestone: M5
---

## Description

Current state: the native Settings window and General, Appearance, Library, Indexer, Services, Privacy, Advanced, and About pages render. Device/data settings are backend-backed; full persistence, sync, updater, encryption, and cross-device coverage remain unverified or incomplete.

Build comprehensive settings pages for configuring libraries, appearance, sync, privacy, and advanced options.

## Pages

- General (app preferences, updates)
- Libraries (manage libraries, encryption)
- Locations (indexed locations, rules)
- Appearance (theme, color scheme, layout)
- Sync (device pairing, sync rules)
- Privacy (telemetry, data sharing)
- Advanced (developer options, debug)
- About (version, credits, licenses)

## Implementation Notes

- Settings stored per-library and globally
- Real-time sync of settings changes
- Form validation with react-hook-form + zod
- Settings categories with sidebar navigation

## Acceptance Criteria

- [x] Settings modal/page opens from menu (`packages/interface/src/routes/settings/index.tsx`)
- [x] Sidebar navigation between categories (`packages/interface/src/Settings/pages/`)
- [ ] General settings (language, updates)
- [ ] Library management (create, delete, encrypt)
- [x] Appearance settings (theme, colors): 12 themes in `AppearanceSettings.tsx` (#201)
- [ ] Sync configuration UI
- [ ] Privacy controls
- [ ] Advanced/debug options
- [ ] All settings persist correctly
- [ ] Settings sync across devices

## WingDrive triage (2026-10-16)

Inherited from upstream Spacedrive. It is not on WingDrive's desktop file-manager path, so it
is parked in the Later milestone M5 (`.tasks/ROADMAP.md`). Re-decide whether WingDrive wants it
before pulling it into a sprint.
