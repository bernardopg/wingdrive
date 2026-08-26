---
id: SETS-000
title: "Epic: Settings Interface"
status: In Progress
assignee: jamiepine
parent: UI-000
priority: Medium
tags: [epic, settings, interface]
whitepaper: N/A
last_updated: 2026-08-24
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

- [ ] Settings modal/page opens from menu
- [ ] Sidebar navigation between categories
- [ ] General settings (language, updates)
- [ ] Library management (create, delete, encrypt)
- [ ] Appearance settings (theme, colors)
- [ ] Sync configuration UI
- [ ] Privacy controls
- [ ] Advanced/debug options
- [ ] All settings persist correctly
- [ ] Settings sync across devices
