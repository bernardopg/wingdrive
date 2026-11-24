---
id: ADAPT-010
title: "Obsidian vault adapter"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S07
tags: [adapters, obsidian, notes]
depends_on: [ADAPT-004, ADAPT-008]
last_updated: 2026-11-24
---

## Description

Index the notes of an Obsidian vault (or any folder of Markdown files): title, body, frontmatter,
tags, wikilinks between notes, created and modified dates. Starts from the prototype in
`adapters/obsidian`. Real data: three vaults on the author's machine (`~/Documentos/Bitter`,
`~/development`, `~/development/ioruba`).

## Implementation Steps

- [ ] Parse YAML frontmatter, inline `#tags`, aliases and `[[wikilinks]]` (with headings and aliases) into note relations
- [ ] Respect `.obsidian` and user exclude patterns; skip attachments but record embeds
- [ ] Detect vaults by their `.obsidian` folder to offer defaults
- [ ] Incremental sync by modified time and content hash; deleted and renamed notes handled

## Acceptance Criteria

- [ ] Lives in `bernardopg/wingdrive-adapters` under `adapters/<id>/`, created from the template (ADAPT-008), with README, license, icon and tests
- [ ] Published through a reviewed pull request that Wingbot's checks (ADAPT-007) pass, like any third-party adapter
- [ ] Installs from the store in the desktop app and from `wing adapters install <id>`
- [ ] Defaults: the source path or account is detected on Linux and macOS (and Windows once ADAPT-002 lands); a wrong value fails `sources.create` with a message that names the field
- [ ] Declares only the permissions it uses (ADAPT-009); runs with network off unless it declares hosts
- [ ] Sync is incremental: a second run with no changes writes nothing new, a changed item is updated in place, and an item removed upstream is removed from the source
- [ ] Items show in Sources > source > items with title, subtitle, date and preview, and are found by the library search
- [ ] Script tests with fixtures cover parsing, cursors, deletions and malformed input, and the contract harness (ADAPT-003) passes in the adapter's CI
- [ ] Real proof on the author's machine: install from the store, sync of real data with item counts, second run idempotent, items opened in the desktop app (screenshot in the PR)
- [ ] Store listing on wingdrive.org/adapters and a user page (EN and PT) with setup, what is indexed and privacy notes
