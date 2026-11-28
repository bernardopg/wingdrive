---
id: ADAPT-010
title: "Obsidian vault adapter"
status: In Progress
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S07
tags: [adapters, obsidian, notes]
depends_on: [ADAPT-004, ADAPT-008]
last_updated: 2026-11-28
---

## Description

Index the notes of an Obsidian vault (or any folder of Markdown files): title, body, frontmatter,
tags, wikilinks between notes, created and modified dates. Starts from the prototype in
`adapters/obsidian`. Real data: three vaults on the author's machine (`~/Documentos/Bitter`,
`~/development`, `~/development/ioruba`).

## Implementation Steps

- [x] Parse YAML frontmatter, inline `#tags`, aliases and `[[wikilinks]]` (with headings and aliases) into note relations (bernardopg/wingdrive-adapters#1; Markdown links too, resolved by path, shortest name, then alias)
- [x] Respect `.obsidian` and user exclude patterns; skip attachments but record embeds (note embeds become links; attachment embeds are skipped)
- [x] Detect vaults by their `.obsidian` folder to offer defaults (bernardopg/wingdrive-adapters#11, 1.1.3: a vault Obsidian lists is offered only while its `.obsidian` folder exists)
- [x] Incremental sync by modified time and size (a hash would mean reading every note each run); deleted and renamed notes handled

## Acceptance Criteria

- [ ] Lives in `bernardopg/wingdrive-adapters` under `adapters/<id>/`, created from the template (ADAPT-008), with README, license, icon and tests
- [x] Published through a reviewed pull request that Wingbot's checks (ADAPT-007) pass, like any third-party adapter
      (1.1.4 through bernardopg/wingdrive-adapters#14: Structure, Code scan, Adapter tests and
      Contract harness pass, labeled `ready-for-review`; published in index 16)
- [ ] Installs from the store in the desktop app and from `wing adapters install <id>`
- [x] Defaults: the source path or account is detected on Linux and macOS (and Windows once ADAPT-002 lands); a wrong value fails `sources.create` with a message that names the field
      (2026-11-28: `sources.create` and `sources.update` now refuse a missing required field and a
      `path_type` folder or file that does not exist, as `config.vault_path`: "Vault: /x is not a
      folder on this device"; before, 3.8.2 created the source and failed at its first sync.
      Detection by `.obsidian` folder is in the registry PR. macOS: 1.1.4,
      bernardopg/wingdrive-adapters#14, tests detection against a fake HOME with
      `~/Library/Application Support/obsidian/obsidian.json`, a local vault and an iCloud vault
      under `~/Library/Mobile Documents`, most recent first; it passes on the registry's `macos-15`
      runners with Python 3.9 and 3.13. Published in index 16.)
- [x] Declares only the permissions it uses (ADAPT-009); runs with network off unless it declares hosts
      (2026-11-28: Wingbot's code scan passes on 1.1.3: no host, no program, no secret, its `~/`
      paths are the declared `obsidian.json` files. It declares no network, so the sandbox gives it
      none; its harness cases pass that way in the registry's Validate run 46165091592)
- [x] Sync is incremental: a second run with no changes writes nothing new, a changed item is updated in place, and an item removed upstream is removed from the source (isolated daemon, 2026-11-25: fixture vault 3 notes, then 2 read, 1 deleted, links fixed; `~/Documentos/Bitter` 50 notes, 10 links, second run 0 read)
- [ ] Items show in Sources > source > items with title, subtitle, date and preview, and are found by the library search
      (2026-11-28, packaged 3.8.2: title, path and preview show, the date did not. `sources.list_items`
      now returns the manifest's `[search] date_field` and the list shows it. The library search
      (`search.files`) does not query sources at all: `ArchiveEngine::search` has no caller in the
      core, so no adapter's items are found there. ADAPT-026 adds it.)
- [x] Script tests with fixtures cover parsing, cursors, deletions and malformed input, and the contract harness (ADAPT-003) passes in the adapter's CI
      (2026-11-28: `fixtures/linked-notes` and `fixtures/missing-vault` pass `wing adapter test`, 2 of 2,
      in bernardopg/wingdrive-adapters#11. Since #13 there the registry's Validate runs the harness on
      them: 2 of 2 in run 46165091592, with the 1.1.3 unit tests.)
- [ ] Real proof on the author's machine: install from the store, sync of real data with item counts, second run idempotent, items opened in the desktop app (screenshot in the PR)
      (2026-11-28, `wingdrive-bin` 3.8.2 on an isolated instance: `wing adapters install obsidian`
      from store index 9, `~/Documentos/Bitter` synced 50 notes, the second run "0 upserted, 0
      deleted", and the desktop app lists the 50 items. Screenshots kept off the PR: they show the
      author's notes. Left: the item dates after this PR ships, and opening an item.)
- [x] Store listing on wingdrive.org/adapters and a user page (EN and PT) with setup, what is indexed and privacy notes
      (2026-11-28: wingdrive.org/adapters/obsidian and /pt/adapters/obsidian list the adapter with install
      steps (app and `wing adapters install obsidian`) and its README, with the Setup, Data and Privacy
      sections in English and Configuração, Dados e Privacidade in Portuguese. The site rebuilds from
      the latest index, so index 16 reaches it at the next build.)
