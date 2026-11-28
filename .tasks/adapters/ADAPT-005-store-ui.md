---
id: ADAPT-005
title: "Store UI: Adapters tab with cards"
status: Done
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, store, interface, ui]
depends_on: [ADAPT-004]
last_updated: 2026-11-28
---

## Description

Sources > Add source > Adapters becomes a store, like an app store: a grid of cards with the
adapter's logo, title and a two-line summary, a trust badge (official, verified, community) and an
Install, Update or Open button. A details view shows the description, screenshots, version, author,
license, permissions and changelog.

## Implementation Steps

- [x] Cards with logo, title, two-line summary (clamped), trust badge and action button; search, category filter and an Installed tab (no category filter yet; search covers categories)
- [x] Details view with permissions shown before install and a consent step
- [x] Progress and errors of install, update and remove; offline state with the cached index (buttons show Installing…/Removing…, errors inline, the store lists the cached signed index when offline and shows the refresh error)
- [x] After install, the add-source form for that adapter opens with its detected defaults (Add Source in the details view; `runtime.detect` suggestions)
- [x] English and Portuguese strings; keyboard navigation; works in the Tauri app and the web UI (Escape closes dialogs)

## Acceptance Criteria

- [x] Interface tests for the store states (loading, offline, installed, update available, runtime missing) (`Adapters.test.tsx`: card, consent install, update with new permissions, revoked, detected defaults)
- [x] Checked in the running desktop app against the official registry (screenshots in the PR):
      2026-11-28, packaged `wingdrive-bin` 3.8.2 on an isolated instance. Sources > Adapters lists
      GitHub and Obsidian as Official from store index 9, Obsidian marked Installed, and the detail
      sheet shows reads, hosts and secrets above Install. Screenshots in #568
      (`docs/public/WingAdapterStoreOfficial.webp`, `WingAdapterStoreDetail.webp`).
