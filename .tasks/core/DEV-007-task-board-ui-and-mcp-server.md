---
id: DEV-007
title: Task board UI and MCP server over .tasks/
status: Done
assignee: bernardopg
parent: DEV-000
priority: High
milestone: M3
sprint: S05
tags: [dev, planning, tooling, mcp]
last_updated: 2026-11-16
---

## Description

Tasks, sprints and the roadmap live only in Markdown, and the author loses track of what is done, in progress and next. The status written in sprint tables drifts from the task front matter, and agents edit front matter by hand without checks.

`tools/board` adds a local kanban UI and an MCP server that read and write the same `.tasks/` files, so the Markdown stays the source of truth and git keeps the history.

## Implementation Steps

- [x] `src/store.ts`: parse tasks, sprints and milestones; minimal line edits to front matter and checkboxes; next free id; integrity check.
- [x] Generated **Board** table in every sprint file and **Progress** table in `ROADMAP.md`, between `board:begin`/`board:end` markers.
- [x] Web UI on 127.0.0.1 (`just board`): columns with drag and drop, filters, sprint strip with WIP, sprints and roadmap views, task drawer, new task dialog, live updates over SSE.
- [x] MCP server (`.mcp.json`): overview, list_tasks, get_task, update_task, set_checklist_item, create_task, sprint_log, check.
- [x] CI and `ci-local` run the board tests and `check --strict`.

## Acceptance Criteria

- [x] Moving a task in the UI changes only its `status` and `last_updated` lines.
- [x] An edit made by an agent or by hand appears in the open board without a reload.
- [x] An MCP client can list, move and log tasks against a copy of `.tasks/` (`bun test`).
- [x] `just board-check` reports zero errors on `main`.
