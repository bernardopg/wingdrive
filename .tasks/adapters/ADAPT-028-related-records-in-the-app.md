---
id: ADAPT-028
title: "Related records of a source item in the app"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: Medium
milestone: M5
sprint: S09
tags: [adapters, interface, archive]
depends_on: [ADAPT-004, ADAPT-005]
related_tasks: [ADAPT-010, ADAPT-012, ADAPT-014]
last_updated: 2026-11-28
---

## Description

Adapters already write relations: Obsidian links notes to notes, GitHub 1.1.0 writes comments that
belong to their issue, and coding-sessions 0.2.0 links sessions to `tool` and `file` records. The
app lists and searches only the primary model, so none of these show anywhere. Until they do,
coding-sessions keeps each message inside the session body instead of writing message records,
which would store the text twice (87,458 messages, 67 MB on the author's laptop) for nothing the
user can see.

## Implementation Steps

- [ ] A library query that returns the records related to one item: `belongs_to` children, the parent, and `many_to_many` links, with each model's title field
- [ ] Sources > source > item shows them in sections named after the model (comments, linked notes, tools, files), each one opening its record
- [ ] Every new UI text through i18n (`en` and `pt`)
- [ ] coding-sessions: `message` records (`belongs_to = ["session"]`, role, position, time, text), written as appended lines arrive, deleted with their session

## Acceptance Criteria

- [ ] On an isolated daemon, a GitHub issue shows its comments, an Obsidian note its links and a coding session its tools and files
- [ ] A record without relations shows no empty sections
