---
id: BRAND-003
title: Wingbot and Wings names in code, settings and the RPC API
status: In Progress
assignee: bernardopg
parent: UI-000
priority: High
milestone: M2
sprint: S05
tags: [branding, core, interface, compatibility]
last_updated: 2026-11-13
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
- [ ] Wings: rename the `Space` entity, `spaces`, `space_items` and `space_groups` tables, the
  `spaces.*` actions and queries, the sync models and the interface code, with a migration and a
  plan for devices on an older version.

## Acceptance Criteria

- [ ] `git grep -i spacebot` and the Space names find only compatibility readers and history.
- [ ] Existing settings, libraries and sidebar collections survive the upgrade.
- [ ] Sync between a new and an older device is either kept or refused with a clear message.

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
