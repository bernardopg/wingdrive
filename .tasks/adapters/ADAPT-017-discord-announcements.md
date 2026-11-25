---
id: ADAPT-017
title: "Adapter announcements on Discord"
status: In Progress
assignee: bernardopg
parent: ADAPT-000
priority: Medium
milestone: M5
sprint: S06
tags: [adapters, discord, automation]
depends_on: [ADAPT-007]
related_tasks: [ADAPT-016]
last_updated: 2026-11-24
---

## Description

The `#adapters` channel in the DEVELOPMENT category of the WingDrive Discord server (created on
2026-11-24) receives one post per published adapter and per adapter update, sent by Wingbot from the
registry's publish workflow.

## Implementation Steps

- [x] Webhook for `#adapters` stored as a secret in the registry's publish environment (`DISCORD_ADAPTERS_WEBHOOK` in `registry-publish`, 2026-11-25)
- [ ] Embed with icon, name, version, two-line summary, author, trust level, permissions, changelog excerpt and links to the store page and the source
- [x] Separate wording for a new adapter, an update, a security fix and a revocation (`wingreg announce`)
- [ ] Channel set so only Wingbot and maintainers post; members can react and open threads

## Acceptance Criteria

- [ ] A test publish posts the expected embed in `#adapters`
- [x] A revocation posts a warning that names the affected versions (unit test in `tests/test_wingreg.py`)
