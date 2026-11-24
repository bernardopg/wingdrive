---
id: ADAPT-000
title: "Epic: Data source adapters"
status: To Do
assignee: bernardopg
priority: High
milestone: M5
sprint: S06
tags: [epic, adapters, sources, archive]
last_updated: 2026-11-24
---

## Description

Sources > Add source > Adapters shows "No adapters installed" on every install. The archive engine,
the source operations (`sources.*`, `adapters.*`) and eleven script adapters under `adapters/` exist,
but no package ships the adapters and the daemon only finds them in the source tree it was built
from.

This epic makes adapters a working feature: adapters reach every package, they share one tested
contract, and the adapters below are finished end to end and proven on real data. Adapters are
scripts that read their config on stdin and print JSONL records (`docs/core/design/archive.md`,
"Adapters").

Decided on 2026-11-24 as an explicit M5 exception, like `INTG-000`: ADAPT-001, ADAPT-003 and the
Obsidian, GitHub, Thunderbird and coding agent session adapters enter S06. The other adapters stay
in the backlog.

| Task | Adapter or work | Plan |
|---|---|---|
| ADAPT-001 | Ship adapters in every package and find them at run time | S06 |
| ADAPT-002 | Python runtime for adapters on Windows | Backlog |
| ADAPT-003 | Shared adapter contract and test harness | S06 |
| ADAPT-010 | Obsidian vaults | S06 |
| ADAPT-011 | Chromium browsers: history and bookmarks | Backlog |
| ADAPT-012 | GitHub: repositories, issues, pull requests, stars | S06 |
| ADAPT-013 | Thunderbird local mail | S06 |
| ADAPT-014 | Coding agent sessions: OpenCode, Claude Code, Codex | S06 |
| ADAPT-020 | Gmail | Backlog |
| ADAPT-021 | Slack export | Backlog |
| ADAPT-022 | Apple Notes | Backlog |
| ADAPT-023 | Safari history | Backlog |
| ADAPT-024 | macOS Calendar | Backlog |
| ADAPT-025 | macOS Contacts | Backlog |

## Acceptance Criteria

- [ ] ADAPT-001 and ADAPT-003 Done
- [ ] ADAPT-010, ADAPT-012, ADAPT-013 and ADAPT-014 Done with their real-data proof
- [ ] A packaged release (AUR and AppImage at least) lists the bundled adapters in Sources > Add source
