# WingDrive Planning

The single source of truth for what to do, in what order, and how far along it is.
If something is not here or in a task under `.tasks/`, it is not planned.

Everything under `.tasks/` is written in English: tasks, sprints, the roadmap, logs and the bug log.

## Method

**Solo Scrumban**: Scrum cadence (short sprints with a goal, review and retro) with Kanban flow
(a WIP limit, pull the next task from the top of the backlog). Small teams and solo developers use
it most because it keeps the plan predictable without big-team ceremony.

- **Two-week** sprints, each with a one-sentence **goal**.
- **WIP limit: 2 tasks** `In Progress` inside the sprint. Finish before starting.
- Roadmap in **Now / Next / Later** form per milestone, with no fixed dates beyond the current sprint.
- Backlog priority: value for daily use as a desktop file manager first; risk of data loss above
  everything.

## Hierarchy

```
Roadmap (ROADMAP.md)
└── Milestone  M1, M2, ...        deliverable outcome (e.g. "Linux desktop file manager 1.0")
    └── Epic   TAURI-000, EXPL-000 task tagged `epic`; groups work by area
        └── Task  TAURI-006, CLI-001 unit of work with acceptance criteria
Sprint (sprints/SNN.md)           time box; pulls tasks from a milestone
Backlog                           `To Do` tasks of a milestone without `sprint`
```

Each task records the hierarchy in its front matter:

```yaml
parent: TAURI-000   # epic
milestone: M1       # roadmap milestone
sprint: S01         # only once committed to a sprint
```

## Naming

Nothing in the project is called "Spacedrive" or uses an `sd`/`sd-` prefix. Use `wingdrive` or
`wing` (crates, binaries, packages, scripts, functions, types, environment variables, text). The
exception is legal attribution to upstream, kept in `CREDITS.md`.

## Definition of Ready (enters the sprint)

- Verifiable acceptance criteria written in the task.
- `milestone` set and dependencies resolved.

## Definition of Done (leaves the sprint)

- Every acceptance criterion checked `[x]` with evidence (a test, a command or a runtime check).
- `cargo fmt`, `cargo clippy -D warnings` and `bun run typecheck` clean for what changed.
- Task at `status: Done` with `last_updated` current; sprint file updated.
- Merged to `main` with a Conventional Commits message.

## Rituals

| When | What | Where to record it |
|---|---|---|
| Sprint start | Planning: goal and tasks pulled from the backlog | `sprints/SNN.md`, `sprint` field of the tasks |
| Every session | Update checkboxes and status of the tasks touched | task file and sprint log |
| Sprint end | Review (what shipped) and retro (what changes) | last section of `sprints/SNN.md` |
| Sprint end | Unfinished tasks go back to the backlog or to the next sprint | `sprint` field |
| Milestone end | Update `ROADMAP.md` (Now/Next/Later) | `ROADMAP.md` |

## Board

`just board` opens the board at <http://127.0.0.1:7357>: To Do, In Progress and Done columns for the
current sprint, drag between columns, filters by sprint, milestone and epic, progress per sprint and
per milestone, and editing of each task's status, sprint and checklist. Agents use the same board
through the `wingdrive-board` MCP server (`.mcp.json`). Both edit the files here, so the `.md` stays
the source of truth. Details in [tools/board/README.md](../tools/board/README.md).

The **Board** table (in each sprint) and the **Progress** table (in the roadmap) are generated from
the front matter between `<!-- board:begin … -->` markers. Do not edit them by hand: `just
board-sync` rebuilds them, and CI fails when they are stale. The rest of the sprint (goal,
commitment order, log, review and retro) is written by hand.

## Commands

```bash
just board-check                                                 # broken references, WIP, tables
just board-sync                                                  # rebuilds the generated tables
cargo run --bin task-validator -- list --milestone M1            # milestone scope
cargo run --bin task-validator -- list --sprint S01              # sprint board
cargo run --bin task-validator -- list --milestone M1 --status "To Do"   # milestone backlog
cargo run --bin task-validator -- validate                       # schema of the staged tasks
```

Planning files (`PLANNING.md`, `ROADMAP.md`, `sprints/SNN.md`) have no hyphen in their names, so
the validator does not treat them as tasks.
