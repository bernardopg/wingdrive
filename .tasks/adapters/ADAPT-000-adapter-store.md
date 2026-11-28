---
id: ADAPT-000
title: "Epic: Adapter store"
status: To Do
assignee: bernardopg
priority: High
milestone: M5
sprint: S06
tags: [epic, adapters, store, sources]
last_updated: 2026-11-24
---

## Description

Sources > Add source > Adapters shows "No adapters installed" on every install. The archive engine,
the `sources.*` and `adapters.*` operations and eleven prototype script adapters under `adapters/`
exist, but nothing reaches an installed app: no package ships the adapters, and the daemon copies
them from `env!("CARGO_MANIFEST_DIR")/../adapters`, a path that only exists on the build machine.

Instead of bundling adapters, WingDrive gets an adapter store, modeled on how DankMaterialShell
handles plugins:

- **Two repositories that work side by side.** `bernardopg/wingdrive` is the app.
  `bernardopg/wingdrive-adapters` is the registry: it holds the signed index, the official adapters
  (`adapters/<id>/`), the templates and the review workflows. Third-party adapters live in their
  authors' repositories and enter the registry by pull request, like a package index.
- **A registry index.** Every published adapter has an entry with a pinned commit and the checksum of
  its archive. The index is signed; the app refuses an index or an archive that
  does not verify. Users can add other registries by URL, like `dms registry add`.
- **Wingbot reviews submissions.** Wingbot, already on Discord, becomes a GitHub App too. A pull
  request to the registry runs automatic checks on structure, requirements, security and tests,
  Wingbot comments one report, and it publishes after a maintainer approves. Each publish posts to
  `#adapters` on Discord.
- **Update notifications.** The app tells the user when WingDrive or an installed adapter has an
  update, with badges, a tray entry and desktop notifications.
- **A store in the app.** The Adapters tab becomes a store: cards with logo, title and a two-line
  description; install, update and remove; the permissions an adapter asks for, shown before install.
- **A catalog on the website.** wingdrive.org/adapters lists the same index.
- **Guidelines for third parties.** A template repository, a `wing adapter` command to create,
  validate, test and pack an adapter, and written rules for review.

The architecture, workflows, checks and diagrams are in
[`docs/design/adapter-store.md`](../../docs/design/adapter-store.md).

Decided on 2026-11-24 as an explicit M5 exception, like `INTG-000`. The store platform enters S06;
the first four official adapters (Obsidian, GitHub, Thunderbird and coding agent sessions) follow in
S07 and go through the same registry as any third party.

| Task | Work | Plan |
|---|---|---|
| ADAPT-001 | Registry repository and signed index format | S06 |
| ADAPT-002 | Python runtime for adapters on Windows | Backlog |
| ADAPT-003 | Adapter contract and test harness | S06 |
| ADAPT-004 | Store backend in the daemon: install, update, remove, lockfile | S06 |
| ADAPT-005 | Store UI: Adapters tab with cards | S06 |
| ADAPT-006 | Catalog at wingdrive.org/adapters | S07 |
| ADAPT-007 | Registry review checks run by Wingbot | S06 |
| ADAPT-008 | Developer guidelines, template repository and `wing adapter` CLI | S06 |
| ADAPT-009 | Adapter permissions and process sandbox | S06 |
| ADAPT-016 | Wingbot GitHub App | S06 |
| ADAPT-017 | Adapter announcements on Discord | S06 |
| ADAPT-018 | Update notifications for adapters and WingDrive | S06 |
| ADAPT-010 | Obsidian vaults | S07 |
| ADAPT-011 | Chromium browsers: history and bookmarks | Backlog |
| ADAPT-012 | GitHub: repositories, issues, pull requests, stars | S07 |
| ADAPT-013 | Thunderbird local mail | S07 |
| ADAPT-014 | Coding agent sessions: OpenCode, Claude Code, Codex | S07 |
| ADAPT-015 | Retire the prototypes in `adapters/` | S07 |
| ADAPT-020 to ADAPT-025 | Gmail, Slack export, Apple Notes, Safari history, macOS Calendar, macOS Contacts | Backlog |
| ADAPT-026 | Library search finds adapter source items | Backlog |

## Acceptance Criteria

- [ ] ADAPT-001, ADAPT-003 to ADAPT-005, ADAPT-007 to ADAPT-009 and ADAPT-016 to ADAPT-018 Done
- [ ] ADAPT-010, ADAPT-012, ADAPT-013 and ADAPT-014 published in the registry and proven on real data
- [ ] A packaged release (AUR and AppImage at least) installs an adapter from the store and syncs it
