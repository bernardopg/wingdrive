// MCP server over stdio: lets agents read and move tasks through the same store the board uses.
// Every call reloads `.tasks/` from disk, so edits made by hand, by the board or by another
// agent are always visible.

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";

import {
	AREAS,
	addSprintLog,
	checkBoard,
	compareTasks,
	createTask,
	getTask,
	loadBoard,
	PRIORITIES,
	STATUSES,
	setChecklistItem,
	summarize,
	syncGenerated,
	type Board,
	type Task,
	updateTask,
	WIP_LIMIT,
	type WriteResult,
} from "./store";

function text(value: string) {
	return { content: [{ type: "text" as const, text: value }] };
}

function fail(error: unknown) {
	return { content: [{ type: "text" as const, text: `error: ${(error as Error).message}` }], isError: true };
}

function row(t: Task): string {
	const check = t.checklist.total ? ` [${t.checklist.done}/${t.checklist.total}]` : "";
	const where = [t.milestone, t.sprint, t.parent && `↑${t.parent}`].filter(Boolean).join(" ");
	return `- ${t.id} (${t.status}, ${t.priority}${where ? `, ${where}` : ""})${check} ${t.title}`;
}

function written(result: WriteResult): string {
	const t = result.task;
	const lines = [`${t.id} saved in ${t.file}`, row(t)];
	for (const w of result.warnings) lines.push(`warning: ${w}`);
	return lines.join("\n");
}

function overview(board: Board): string {
	const lines: string[] = [`Today ${board.today}.`];
	const sprint = board.sprints.find((s) => s.id === board.currentSprint);
	if (sprint) {
		const tasks = board.tasks.filter((t) => t.sprint === sprint.id).sort(compareTasks);
		const c = summarize(tasks);
		const wip = tasks.filter((t) => t.status === "In Progress" && !t.isEpic).length;
		lines.push(
			"",
			`## Current sprint ${sprint.id} (${sprint.start} to ${sprint.end})`,
			sprint.goal ? `Goal: ${sprint.goal}` : "",
			`${c.done}/${c.total} Done, ${c.doing} In Progress (WIP ${wip}/${WIP_LIMIT} excluding epics), ${c.todo} To Do`,
			...tasks.filter((t) => t.status !== "Done").map(row),
		);
	} else {
		lines.push("No sprint covers today; plan one in .tasks/sprints/.");
	}
	lines.push("", "## Milestones");
	for (const m of board.milestones) {
		const c = summarize(board.tasks.filter((t) => t.milestone === m.id));
		lines.push(`- ${m.id} ${m.phase}: ${m.title} — ${c.done}/${c.total} Done, ${c.doing} In Progress`);
	}
	const issues = checkBoard(board);
	const errors = issues.filter((i) => i.level === "error");
	lines.push("", `Integrity: ${errors.length} errors, ${issues.length - errors.length} warnings (run \`check\`).`);
	return lines.filter((l, i, a) => !(l === "" && a[i - 1] === "")).join("\n");
}

const status = z.enum(STATUSES);
const priority = z.enum(PRIORITIES);

export async function runMcp(): Promise<void> {
	const server = new McpServer(
		{ name: "wingdrive-board", version: "0.1.0" },
		{
			instructions:
				"WingDrive task board over .tasks/. Markdown files are the source of truth; these tools edit them " +
				"in place and regenerate the sprint and roadmap tables. Start with `overview`. When you start a task, " +
				"set it In Progress; when you finish it, check its items with evidence, set it Done and add a sprint log " +
				"line. Respect the WIP limit of 2 tasks In Progress per sprint. Never mark Done without the acceptance " +
				"criteria checked.",
		},
	);

	server.registerTool(
		"overview",
		{
			title: "Board overview",
			description: "Current sprint with its open tasks, WIP, milestone progress and integrity counts.",
			annotations: { readOnlyHint: true },
		},
		async () => {
			try {
				return text(overview(loadBoard()));
			} catch (e) {
				return fail(e);
			}
		},
	);

	server.registerTool(
		"list_tasks",
		{
			title: "List tasks",
			description:
				"Tasks filtered by status, sprint ('current' allowed), milestone, parent epic, tag, area or a text query " +
				"on id and title. Sorted In Progress, To Do, Done, then by priority.",
			inputSchema: {
				status: status.optional(),
				sprint: z.string().optional().describe("S05, or 'current'"),
				milestone: z.string().optional().describe("M3"),
				parent: z.string().optional().describe("epic id, e.g. DESK-000"),
				tag: z.string().optional(),
				area: z.enum(AREAS).optional(),
				query: z.string().optional(),
				limit: z.number().int().positive().max(400).optional().describe("default 60"),
			},
			annotations: { readOnlyHint: true },
		},
		async (args) => {
			try {
				const board = loadBoard();
				const sprint = args.sprint === "current" ? board.currentSprint : args.sprint?.toUpperCase();
				const q = args.query?.toLowerCase();
				const tasks = board.tasks
					.filter(
						(t) =>
							(!args.status || t.status === args.status) &&
							(!sprint || t.sprint === sprint) &&
							(!args.milestone || t.milestone === args.milestone.toUpperCase()) &&
							(!args.parent || t.parent === args.parent.toUpperCase()) &&
							(!args.tag || t.tags.includes(args.tag)) &&
							(!args.area || t.area === args.area) &&
							(!q || t.id.toLowerCase().includes(q) || t.title.toLowerCase().includes(q)),
					)
					.sort(compareTasks);
				const limit = args.limit ?? 60;
				const shown = tasks.slice(0, limit).map(row);
				if (tasks.length > limit) shown.push(`… ${tasks.length - limit} more; raise limit or narrow the filter`);
				return text(tasks.length ? `${tasks.length} tasks\n${shown.join("\n")}` : "no tasks match");
			} catch (e) {
				return fail(e);
			}
		},
	);

	server.registerTool(
		"get_task",
		{
			title: "Get task",
			description: "Front matter, file path, numbered checklist and full Markdown body of one task.",
			inputSchema: { id: z.string() },
			annotations: { readOnlyHint: true },
		},
		async ({ id }) => {
			try {
				const t = getTask(loadBoard(), id);
				const meta = {
					id: t.id,
					title: t.title,
					status: t.status,
					priority: t.priority,
					assignee: t.assignee,
					parent: t.parent,
					milestone: t.milestone,
					sprint: t.sprint,
					tags: t.tags,
					last_updated: t.lastUpdated,
					related_tasks: t.related,
					file: t.file,
				};
				const items = t.items.map((i) => `${i.index}. [${i.done ? "x" : " "}] ${i.text}`).join("\n");
				return text(
					`${JSON.stringify(meta, null, 2)}\n\nChecklist (index for set_checklist_item):\n${items || "(none)"}\n\n---\n${t.body}`,
				);
			} catch (e) {
				return fail(e);
			}
		},
	);

	server.registerTool(
		"update_task",
		{
			title: "Update task",
			description:
				"Changes front matter fields of a task and sets last_updated to today. Pass null for sprint or parent " +
				"to clear it. Returns warnings for WIP over the limit or Done with unchecked items.",
			inputSchema: {
				id: z.string(),
				status: status.optional(),
				priority: priority.optional(),
				sprint: z.string().nullable().optional(),
				milestone: z.string().optional(),
				parent: z.string().nullable().optional(),
				assignee: z.string().optional(),
				title: z.string().optional(),
				tags: z.array(z.string()).optional(),
			},
			annotations: { destructiveHint: false, idempotentHint: true },
		},
		async ({ id, ...patch }) => {
			try {
				return text(written(updateTask(loadBoard(), id, patch)));
			} catch (e) {
				return fail(e);
			}
		},
	);

	server.registerTool(
		"set_checklist_item",
		{
			title: "Check a checklist item",
			description: "Checks or unchecks one checkbox of a task by its index from get_task.",
			inputSchema: { id: z.string(), index: z.number().int().min(0), done: z.boolean() },
			annotations: { destructiveHint: false, idempotentHint: true },
		},
		async ({ id, index, done }) => {
			try {
				const result = setChecklistItem(loadBoard(), id, index, done);
				const item = result.task.items[index];
				return text(`${written(result)}\n${index}. [${item.done ? "x" : " "}] ${item.text}`);
			} catch (e) {
				return fail(e);
			}
		},
	);

	server.registerTool(
		"create_task",
		{
			title: "Create task",
			description:
				"Creates .tasks/<area>/<ID>-<slug>.md with the next free id for the prefix. Every task needs a milestone; " +
				"give steps and verifiable acceptance criteria.",
			inputSchema: {
				prefix: z.string().describe("CORE, DESK, TAURI, MOB, …"),
				title: z.string().min(5),
				area: z.enum(AREAS),
				milestone: z.string(),
				priority: priority.optional(),
				status: status.optional(),
				parent: z.string().optional(),
				sprint: z.string().optional(),
				tags: z.array(z.string()).optional(),
				description: z.string().optional(),
				steps: z.array(z.string()).optional(),
				criteria: z.array(z.string()).optional(),
			},
			annotations: { destructiveHint: false },
		},
		async (input) => {
			try {
				return text(written(createTask(loadBoard(), input)));
			} catch (e) {
				return fail(e);
			}
		},
	);

	server.registerTool(
		"sprint_log",
		{
			title: "Add sprint log line",
			description: "Adds '- <today>: <text>' at the top of the sprint's Log section. Defaults to the current sprint.",
			inputSchema: { text: z.string().min(3), sprint: z.string().optional() },
			annotations: { destructiveHint: false },
		},
		async ({ text: line, sprint }) => {
			try {
				const board = loadBoard();
				const id = sprint?.toUpperCase() ?? board.currentSprint;
				if (!id) throw new Error("no current sprint; pass sprint");
				return text(`${id}: ${addSprintLog(board, id, line)}`);
			} catch (e) {
				return fail(e);
			}
		},
	);

	server.registerTool(
		"check",
		{
			title: "Check integrity",
			description:
				"Finds broken references (parent, sprint, milestone), invalid fields, duplicate ids, open tasks without " +
				"sprint, Done tasks with unchecked items and WIP over the limit. Also refreshes the generated tables.",
			inputSchema: { level: z.enum(["error", "warn"]).optional().describe("error = errors only") },
		},
		async ({ level }) => {
			try {
				const board = loadBoard();
				const changed = syncGenerated(board);
				const issues = checkBoard(board).filter((i) => level !== "error" || i.level === "error");
				const lines = issues.map((i) => `${i.level}: ${i.id ? `${i.id} ` : ""}${i.message}`);
				if (changed.length) lines.push(`regenerated: ${changed.join(", ")}`);
				return text(lines.length ? lines.join("\n") : "no issues");
			} catch (e) {
				return fail(e);
			}
		},
	);

	await server.connect(new StdioServerTransport());
}
