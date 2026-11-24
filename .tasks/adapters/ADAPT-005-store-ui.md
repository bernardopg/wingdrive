---
id: ADAPT-005
title: "Store UI: Adapters tab with cards"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, store, interface, ui]
depends_on: [ADAPT-004]
last_updated: 2026-11-24
---

## Description

Sources > Add source > Adapters becomes a store, like an app store: a grid of cards with the
adapter's logo, title and a two-line summary, a trust badge (official, verified, community) and an
Install, Update or Open button. A details view shows the description, screenshots, version, author,
license, permissions and changelog.

## Implementation Steps

- [ ] Cards with logo, title, two-line summary (clamped), trust badge and action button; search, category filter and an Installed tab
- [ ] Details view with permissions shown before install and a consent step
- [ ] Progress and errors of install, update and remove; offline state with the cached index
- [ ] After install, the add-source form for that adapter opens with its detected defaults
- [ ] English and Portuguese strings; keyboard navigation; works in the Tauri app and the web UI

## Acceptance Criteria

- [ ] Interface tests for the store states (loading, offline, installed, update available, runtime missing)
- [ ] Checked in the running desktop app against the official registry (screenshots in the PR)
