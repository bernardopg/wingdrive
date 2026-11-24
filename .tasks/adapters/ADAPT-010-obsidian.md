---
id: ADAPT-010
title: "Obsidian vault adapter"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, obsidian, notes]
related_tasks: [ADAPT-003]
last_updated: 2026-11-24
---

## Description

Index the notes of an Obsidian vault (or any folder of Markdown files): title, body, frontmatter,
tags, wikilinks between notes, created and modified dates. The current `adapters/obsidian` is a
prototype without tests. Real data: three vaults on the author's machine
(`~/Documentos/Bitter`, `~/development`, `~/development/ioruba`).

## Implementation Steps

- [ ] Parse YAML frontmatter, inline `#tags`, aliases and `[[wikilinks]]` (with headings and aliases) into note relations
- [ ] Respect `.obsidian` and user exclude patterns; skip attachments but record embeds
- [ ] Detect vaults by their `.obsidian` folder to offer defaults
- [ ] Incremental sync by modified time and content hash; deleted and renamed notes handled

## Acceptance Criteria

- [ ] Defaults: the source path or account is detected on Linux and macOS (and Windows once ADAPT-002 lands); a wrong value fails `sources.create` with a message that names the field
- [ ] Sync is incremental: a second run with no changes writes nothing new, a changed item is updated in place, and an item removed upstream is removed from the source
- [ ] Items show in Sources > source > items with title, subtitle, date and preview, and are found by the library search
- [ ] Script tests with fixtures cover parsing, cursors, deletions and malformed input (`python3 -m unittest` under the adapter folder, run in CI)
- [ ] Rust integration test runs the real `sync.py` through `ScriptAdapter` against the fixtures and checks the stored records
- [ ] Real proof on the author's machine: sync of real data with item counts, second run idempotent, items opened in the desktop app (screenshot in the PR)
- [ ] Docs: a page in `docs/` and in the wingdrive.org user guide (EN and PT) with setup, what is indexed and privacy notes
