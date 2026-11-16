---
id: BRAND-003
title: Wingbot and Wings names in code, settings and the RPC API
status: Done
assignee: bernardopg
parent: UI-000
priority: High
milestone: M2
sprint: S05
tags: [branding, core, interface, compatibility]
last_updated: 2026-11-15
---

## Description

BRAND-002 renamed what the user reads and kept `spacebot` and `spaces` in code, settings, the RPC
API and the database. On 2026-11-06 the user asked for the WingDrive identity there too: no
`Spacebot`, `Spacedrive`, `sd` or `space` names. Technical words such as whitespace, namespace,
workspace, color space and free space stay. Upstream attribution required by the license stays.

Two pull requests: Wingbot first, then Wings, which needs a database migration and a sync-protocol
change.

## Implementation Steps

- [x] Wingbot: `WingbotConfig`, the `wingbot` settings key and `wingbot_*` RPC fields, generated
  TypeScript, the `@wingbot/api-client` alias and its stub, `VITE_WINGBOT_AVAILABLE`, docs in
  `docs/core/design/wingbot-*.md`, and a code-wide check in `scripts/check-wingdrive-independence.sh`.
- [x] Wingbot: settings saved under `spacebot` are still read and saved back as `wingbot`; saved tab
  paths and window labels `spacebot` still open Wingbot.
- [x] `sdPath` and its variants (`sdPaths`, `sdPathToUri`, `sd_path`, `*_sdpath`) renamed to
  `wdPath` across core, CLI, desktop, mobile and the interface (2026-11-13).
- [x] Wings: the `Wing`, `WingGroup`, `WingItem` and `WingLayout` types, the `wings`, `wing_groups`
  and `wing_items` tables (migration `m20261115_000001_rename_spaces_to_wings`), the `wings.*`
  actions and queries, the `wing wings` CLI command (`spaces` stays an alias), `WingsSidebar`,
  `useWings` and the mobile Browse screen (2026-11-15).
- [x] Wings: sync keeps the model names `space`, `space_group` and `space_item`, the `space_id` and
  `space_uuid` fields and the seeds of the deterministic default UUIDs, so devices on 3.3.1 and
  older keep syncing. `resource_type_for_sync_model` maps those model names to the `wing*`
  resource types. The sidebar store moves a saved `currentSpaceId` to `currentWingId`.
- [x] Wings: code-wide check in `scripts/check-wingdrive-independence.sh`.
- [x] Wings: checked against a copy of the author's library (2026-11-15).
- [x] Wings: sidebar checked in the desktop app and sync with a device on 3.3.1 (2026-11-15).

## Acceptance Criteria

- [x] `git grep -i spacebot` and the Space names find only compatibility readers and history.
- [x] Existing settings, libraries and sidebar collections survive the upgrade.
- [x] Sync between a new and an older device is either kept or refused with a clear message.
  Kept by design: the wire names did not change (`wing_layout_test` checks them).

## Evidence

- 2026-11-06, Wingbot: a real `wingdrive.json` with the `spacebot` key was read by the daemon and
  saved back as `wingbot`; `/spacebot/chat` in the desktop app redirects to `/wingbot/chat`.
  Details in [COV-02](../testing/COV-02-contract-regression-tests.md).
- 2026-11-13, inventory after the `wdPath` rename: no `sd`-prefixed identifiers remain except
  `SDMEMORY` (the memory archive magic bytes, renaming it breaks saved files), `SD_Light` (the SD-card
  icon) and the `sd_core` log filter migration in `app_config.rs`. `Spacebot` and `Spacedrive` appear
  only in compatibility readers, legacy path adoption (`wing-branding`) and upstream attribution.
  The Space names are the remaining work: about 170 `space_id`, 77 `SpaceItem`, 73 `space_group`,
  plus `SpaceLayout`, `SpacesSidebar`, `useSpaces`, the `spaces.*` wire methods, the `spaces`,
  `space_items` and `space_groups` tables and the sync models. Planned in S05.
- 2026-11-15, Wings: the migration test builds the 3.3.1 schema with `Migrator::down`, saves a wing
  with a group and two items, migrates and finds every row, the links and foreign keys under the new
  names; a failed rename leaves the old tables. `wing_layout_test` (4), `wing_contract_test`,
  `sync_setup_test`, the sync registry tests and `sidebar-store.test.ts` pass.
- 2026-11-15, the author's library (`sqlite3 .backup` of `library.db`, `sync.db` and `jobs.db`, the
  original untouched) opened by the new daemon in an isolated instance: the migration ran, `wings`,
  `wing_groups` and `wing_items` hold the 1 wing, 5 groups and 6 items of before,
  `PRAGMA foreign_key_check` is empty, `wing wings list` and `wing wings layout` show them, and no
  second default wing was created. A unit test pins `space_id`, `space_uuid` and the three model
  names on the wire.
- 2026-11-15, sync with a device on `v3.3.1`: a daemon built from `main` (named instance) paired
  with the moto g(7) on `v3.3.1` and shared a new library. A wing ("Trabalho"), a group ("Projetos")
  and two items created on the desktop through `wings.create`, `wings.add_group` and
  `wings.add_item` reached the phone in seconds; its log shows "Shared change applied successfully"
  for `space_group` and `space_item`, and Explore shows the wing and the group. The default groups
  the phone created for the shared library have the same deterministic ids as the desktop's, so
  the desktop still lists one "All Devices" wing with five groups. The phone hides Recents and
  Favorites items by design (MOB-007); its review is MOB-017.
- 2026-11-15, desktop sidebar: the same interface the Tauri app serves (`wing-server`, release
  build) against that daemon. The wing switcher lists "All Devices" and "Trabalho" with "New Wing"
  and "Wing Settings"; "Trabalho" shows Favorites, the Projetos group with Recents, and Folders. The
  sidebar store saved `currentWingId` at version 1, and the page logged no error.
