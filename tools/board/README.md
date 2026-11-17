# WingDrive board

Kanban board and MCP server for the task tracker in `.tasks/`. The Markdown files stay the source of
truth: the board and the agents edit the same files, in place, with the smallest possible diff, and
every change shows up in git.

```bash
just board            # UI on http://127.0.0.1:7357 (--port N, --open)
just board-sync       # rebuild the progress tables in the sprint files and ROADMAP.md
just board-check      # broken references, invalid fields, WIP over the limit, stale tables
```

The first run installs the dependencies (`bun install --cwd tools/board`).

## Views

- **Quadro**: To Do, In Progress and Done columns for the current sprint by default. Drag a task to
  another column, or focus it and press Alt+Right / Alt+Left. Filter by sprint, milestone, epic,
  priority and area; `/` focuses the search. Filters are kept per browser.
- **Sprints**: every sprint with its dates, goal and progress. Click one to see its board.
- **Roadmap**: milestones by phase (Now, Next, Later, Done) with progress per epic.
- **Task drawer**: click a task to edit status, priority, sprint and milestone, and to check
  checklist items. Links to the epic, its children and related tasks.
- **Nova task** (`n`): creates `.tasks/<area>/<ID>-<slug>.md` with the next free id of the prefix.

The board watches `.tasks/` and updates live when an agent, an editor or `git pull` changes a file.
Keys: `1` `2` `3` switch views, `Esc` closes the drawer.

The server binds to `127.0.0.1` only and refuses cross-origin writes.

## What the board writes

- Front matter keys it changes (`status`, `sprint`, …) plus `last_updated`, by line edits that keep
  the rest of the front matter, comments included.
- Checkbox lines (`- [ ]` / `- [x]`) in the task body; code blocks are skipped.
- The generated blocks between `<!-- board:begin … -->` and `<!-- board:end … -->`: the **Quadro**
  table in each sprint file and the **Progresso** table in `ROADMAP.md`. Do not edit them by hand;
  `just board-sync` rewrites them, and CI fails when they are stale.
- `## Log` lines in the sprint file (MCP `sprint_log`).

The rest of the sprint files and the roadmap (goal, commitment order, review, retro, history) stays
hand-written.

## MCP server for agents

`.mcp.json` at the repository root registers `wingdrive-board` for Claude Code. Other agents run the
same stdio server:

```bash
# Codex
codex mcp add wingdrive-board --env BOARD_ROOT=/path/to/wingdrive -- bun /path/to/wingdrive/tools/board/src/cli.ts mcp
```

`BOARD_ROOT` is only needed when the agent does not start in the repository.

| Tool | What it does |
|---|---|
| `overview` | Current sprint, its open tasks, WIP, milestone progress and integrity counts |
| `list_tasks` | Filter by status, sprint (`current`), milestone, epic, tag, area or text |
| `get_task` | Front matter, numbered checklist and the full Markdown body |
| `update_task` | Status, priority, sprint, milestone, parent, title, tags; stamps `last_updated` |
| `set_checklist_item` | Check or uncheck one item by index |
| `create_task` | New task file with the next id, steps and acceptance criteria |
| `sprint_log` | Dated line at the top of the sprint's `## Log` |
| `check` | Integrity report; also refreshes the generated tables |

Writes return warnings when a sprint goes over the WIP limit of 2 or a task becomes Done with
unchecked items, the two rules from `.tasks/PLANNING.md` that drift most.

## Development

```bash
cd tools/board
bun test             # store unit tests and an end-to-end MCP test on a copy of .tasks/
bun run typecheck
bun src/cli.ts serve --dev   # hot reload for the UI
```

`src/store.ts` holds all parsing and writing; the HTTP server (`src/server.ts`), the MCP server
(`src/mcp.ts`) and the CLI only call it. The UI is React in `ui/`, bundled by Bun at startup.
