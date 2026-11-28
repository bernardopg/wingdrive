// Reads and writes the `.tasks/` tracker. The Markdown files stay the source of truth: every
// change made here is a minimal line edit to a task's front matter or body, so diffs stay small
// and humans, agents and the board all edit the same files.

import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import YAML from "yaml";

export const STATUSES = ["To Do", "In Progress", "Done"] as const;
export type Status = (typeof STATUSES)[number];
export const PRIORITIES = ["High", "Medium", "Low"] as const;
export type Priority = (typeof PRIORITIES)[number];
export const AREAS = ["core", "interface", "mobile", "testing", "adapters"] as const;
export type Area = (typeof AREAS)[number];
export type Phase = "Done" | "Now" | "Next" | "Later";

/** WIP limit per sprint, from PLANNING.md ("WIP máximo: 2 tasks In Progress"). */
export const WIP_LIMIT = 2;

export interface ChecklistItem {
	index: number;
	done: boolean;
	text: string;
}

export interface Task {
	id: string;
	title: string;
	status: Status;
	priority: Priority;
	assignee: string;
	parent?: string;
	milestone?: string;
	sprint?: string;
	tags: string[];
	lastUpdated?: string;
	related: string[];
	dependsOn: string[];
	area: string;
	/** Path relative to the repository root. */
	file: string;
	isEpic: boolean;
	checklist: { done: number; total: number };
	/** Raw status when it is not one of STATUSES (reported by `check`). */
	rawStatus?: string;
	rawPriority?: string;
}

export interface TaskDetail extends Task {
	body: string;
	items: ChecklistItem[];
}

export interface Sprint {
	id: string;
	number: number;
	start?: string;
	end?: string;
	goal?: string;
	milestones?: string;
	file: string;
}

export interface Milestone {
	id: string;
	phase: Phase;
	title: string;
}

export interface Board {
	root: string;
	today: string;
	tasks: Task[];
	sprints: Sprint[];
	milestones: Milestone[];
	currentSprint?: string;
	/** Task files whose front matter could not be parsed. */
	broken: { file: string; message: string }[];
}

export interface Issue {
	level: "error" | "warn";
	id?: string;
	file?: string;
	message: string;
}

const FRONT_MATTER = /^---\r?\n([\s\S]*?)\r?\n---[ \t]*(?:\r?\n|$)/;
const CHECKBOX = /^(\s*[-*+] \[)( |x|X)(\] ?)(.*)$/;
const FENCE = /^\s*(```|~~~)/;
const STATUS_ALIASES: Record<string, Status> = { Completed: "Done", Complete: "Done" };

export function today(): string {
	return process.env.BOARD_TODAY ?? new Date().toLocaleDateString("sv-SE");
}

/** Finds the repository root: `BOARD_ROOT`, else the first ancestor of cwd or of this file with `.tasks/`. */
export function findRoot(): string {
	if (process.env.BOARD_ROOT) return resolve(process.env.BOARD_ROOT);
	for (const start of [process.cwd(), import.meta.dir]) {
		let dir = resolve(start);
		while (true) {
			if (existsSync(join(dir, ".tasks", "PLANNING.md"))) return dir;
			const up = dirname(dir);
			if (up === dir) break;
			dir = up;
		}
	}
	throw new Error("no .tasks/PLANNING.md found above the working directory; set BOARD_ROOT");
}

function tasksDir(root: string): string {
	return join(root, ".tasks");
}

function asList(value: unknown): string[] {
	if (Array.isArray(value)) return value.map(String);
	if (typeof value === "string" && value.trim()) return value.split(",").map((s) => s.trim());
	return [];
}

function asString(value: unknown): string | undefined {
	if (value === null || value === undefined || value === "") return undefined;
	return String(value);
}

export function splitFrontMatter(text: string): { data: Record<string, unknown>; body: string } | null {
	const match = FRONT_MATTER.exec(text);
	if (!match) return null;
	const data = YAML.parse(match[1]) as Record<string, unknown> | null;
	if (!data || typeof data !== "object") return null;
	return { data, body: text.slice(match[0].length) };
}

/** Checkbox lines of a body, skipping fenced code blocks. */
export function checklistItems(body: string): ChecklistItem[] {
	const items: ChecklistItem[] = [];
	let fenced = false;
	for (const line of body.split("\n")) {
		if (FENCE.test(line)) {
			fenced = !fenced;
			continue;
		}
		if (fenced) continue;
		const m = CHECKBOX.exec(line);
		if (m) items.push({ index: items.length, done: m[2] !== " ", text: m[4].trim() });
	}
	return items;
}

function parseTask(root: string, path: string): TaskDetail | null {
	const text = readFileSync(path, "utf8");
	const parsed = splitFrontMatter(text);
	if (!parsed || !parsed.data.id) return null;
	const d = parsed.data;
	const rawStatus = String(d.status ?? "");
	const status = (STATUS_ALIASES[rawStatus] ?? rawStatus) as Status;
	const rawPriority = String(d.priority ?? "");
	const tags = asList(d.tags);
	const items = checklistItems(parsed.body);
	const id = String(d.id);
	const rel = relative(root, path);
	return {
		id,
		title: String(d.title ?? id),
		status: STATUSES.includes(status) ? status : "To Do",
		rawStatus: STATUSES.includes(status) ? undefined : rawStatus,
		priority: PRIORITIES.includes(rawPriority as Priority) ? (rawPriority as Priority) : "Medium",
		rawPriority: PRIORITIES.includes(rawPriority as Priority) ? undefined : rawPriority,
		assignee: String(d.assignee ?? ""),
		parent: asString(d.parent),
		milestone: asString(d.milestone),
		sprint: asString(d.sprint),
		tags,
		lastUpdated: asString(d.last_updated),
		related: asList(d.related_tasks),
		dependsOn: [...asList(d.depends_on), ...asList(d.dependencies)],
		area: rel.split("/")[1] ?? "",
		file: rel,
		isEpic: tags.includes("epic"),
		checklist: { done: items.filter((i) => i.done).length, total: items.length },
		body: parsed.body,
		items,
	};
}

function taskFiles(root: string): string[] {
	const files: string[] = [];
	const base = tasksDir(root);
	for (const entry of readdirSync(base)) {
		const dir = join(base, entry);
		if (entry === "sprints" || !statSync(dir).isDirectory()) continue;
		for (const name of readdirSync(dir)) {
			if (name.endsWith(".md") && name.includes("-")) files.push(join(dir, name));
		}
	}
	return files.sort();
}

function parseSprint(root: string, path: string): Sprint | null {
	const text = readFileSync(path, "utf8");
	const id = basename(path, ".md");
	if (!/^S\d+$/.test(id)) return null;
	const head = /^# Sprint S\d+\s*[—–-]\s*(\d{4}-\d{2}-\d{2})\s+(?:to|a)\s+(\d{4}-\d{2}-\d{2})/m.exec(text);
	const goal = /^\*\*(?:Goal|Meta):\*\*\s*([\s\S]*?)(?=\n\s*\n|(?![\s\S]))/m.exec(text);
	const milestones = /^Milestones?:\s*(.+)$/m.exec(text);
	return {
		id,
		number: Number(id.slice(1)),
		start: head?.[1],
		end: head?.[2],
		goal: goal?.[1].replace(/\s*\n\s*/g, " ").trim(),
		milestones: milestones?.[1].replace(/\*\*/g, "").trim(),
		file: relative(root, path),
	};
}

export function parseMilestones(roadmap: string): Milestone[] {
	const out: Milestone[] = [];
	const re = /^## (Done|Now|Next|Later) [—–-] (M\d+): (.+)$/gm;
	for (let m = re.exec(roadmap); m; m = re.exec(roadmap)) {
		out.push({ id: m[2], phase: m[1] as Phase, title: m[3].trim() });
	}
	return out.sort((a, b) => Number(a.id.slice(1)) - Number(b.id.slice(1)));
}

export function loadBoard(root = findRoot()): Board {
	const broken: Board["broken"] = [];
	const tasks = taskFiles(root)
		.map((f) => {
			try {
				return parseTask(root, f);
			} catch (e) {
				broken.push({ file: relative(root, f), message: (e as Error).message.split("\n")[0] });
				return null;
			}
		})
		.filter((t): t is TaskDetail => t !== null)
		.map(({ body: _body, items: _items, ...task }) => task);
	const sprintDir = join(tasksDir(root), "sprints");
	const sprints = existsSync(sprintDir)
		? readdirSync(sprintDir)
				.filter((n) => n.endsWith(".md"))
				.map((n) => parseSprint(root, join(sprintDir, n)))
				.filter((s): s is Sprint => s !== null)
				.sort((a, b) => a.number - b.number)
		: [];
	const roadmapPath = join(tasksDir(root), "ROADMAP.md");
	const milestones = existsSync(roadmapPath) ? parseMilestones(readFileSync(roadmapPath, "utf8")) : [];
	const day = today();
	const currentSprint = sprints.find((s) => s.start && s.end && s.start <= day && day <= s.end)?.id;
	return { root, today: day, tasks, sprints, milestones, currentSprint, broken };
}

export function getTask(board: Board, id: string): TaskDetail {
	const task = board.tasks.find((t) => t.id.toUpperCase() === id.toUpperCase());
	if (!task) throw new Error(`task ${id} not found`);
	const detail = parseTask(board.root, join(board.root, task.file));
	if (!detail) throw new Error(`task ${id} has no front matter`);
	return detail;
}

// ---------------------------------------------------------------------------------------------
// Writing

function formatScalar(value: string): string {
	if (/[,[\]{}]/.test(value)) return JSON.stringify(value);
	return YAML.stringify(value, { lineWidth: 0 }).trimEnd();
}

function formatValue(value: string | string[]): string {
	return Array.isArray(value) ? `[${value.map(formatScalar).join(", ")}]` : formatScalar(value);
}

/**
 * Sets (or with `null`, removes) top-level front matter keys with line edits, keeping every
 * other line, key order and comment as it was.
 */
export function setFrontMatter(text: string, entries: [string, string | string[] | null][]): string {
	const match = FRONT_MATTER.exec(text);
	if (!match) throw new Error("file has no front matter");
	const lines = match[1].split(/\r?\n/);
	for (const [key, value] of entries) {
		const at = lines.findIndex((l) => l.startsWith(`${key}:`));
		let end = at + 1;
		if (at >= 0) while (end < lines.length && /^(\s|- )/.test(lines[end])) end++;
		if (value === null) {
			if (at >= 0) lines.splice(at, end - at);
		} else if (at >= 0) {
			lines.splice(at, end - at, `${key}: ${formatValue(value)}`);
		} else {
			lines.push(`${key}: ${formatValue(value)}`);
		}
	}
	return `---\n${lines.join("\n")}\n---\n${text.slice(match[0].length)}`;
}

export interface TaskPatch {
	title?: string;
	status?: Status;
	priority?: Priority;
	assignee?: string;
	parent?: string | null;
	milestone?: string;
	sprint?: string | null;
	tags?: string[];
}

export interface WriteResult {
	task: TaskDetail;
	warnings: string[];
}

function validatePatch(board: Board, patch: TaskPatch): void {
	if (patch.status && !STATUSES.includes(patch.status)) throw new Error(`invalid status ${patch.status}`);
	if (patch.priority && !PRIORITIES.includes(patch.priority)) {
		throw new Error(`invalid priority ${patch.priority}`);
	}
	if (patch.sprint && !board.sprints.some((s) => s.id === patch.sprint)) {
		throw new Error(`sprint ${patch.sprint} has no file in .tasks/sprints/`);
	}
	if (patch.milestone && !board.milestones.some((m) => m.id === patch.milestone)) {
		throw new Error(`milestone ${patch.milestone} is not in ROADMAP.md`);
	}
	if (patch.parent && !board.tasks.some((t) => t.id === patch.parent)) {
		throw new Error(`parent ${patch.parent} not found`);
	}
	if (patch.title !== undefined && patch.title.trim().length < 5) throw new Error("title needs 5+ characters");
}

function wipWarnings(board: Board, task: TaskDetail): string[] {
	if (task.status !== "In Progress" || !task.sprint) return [];
	const wip = board.tasks.filter(
		(t) => t.sprint === task.sprint && t.status === "In Progress" && !t.isEpic && t.id !== task.id,
	).length + (task.isEpic ? 0 : 1);
	return wip > WIP_LIMIT ? [`${task.sprint} has ${wip} tasks In Progress (WIP limit ${WIP_LIMIT})`] : [];
}

export function updateTask(board: Board, id: string, patch: TaskPatch): WriteResult {
	const current = getTask(board, id);
	validatePatch(board, patch);
	const entries: [string, string | string[] | null][] = [];
	const keys: [keyof TaskPatch, string][] = [
		["title", "title"],
		["status", "status"],
		["priority", "priority"],
		["assignee", "assignee"],
		["parent", "parent"],
		["milestone", "milestone"],
		["sprint", "sprint"],
		["tags", "tags"],
	];
	for (const [field, key] of keys) {
		if (patch[field] === undefined) continue;
		const value = patch[field];
		entries.push([key, value === null || value === "" ? null : (value as string | string[])]);
	}
	if (entries.length === 0) return { task: current, warnings: [] };
	entries.push(["last_updated", board.today]);
	const path = join(board.root, current.file);
	writeFileSync(path, setFrontMatter(readFileSync(path, "utf8"), entries));
	const fresh = loadBoard(board.root);
	const task = getTask(fresh, current.id);
	const warnings = wipWarnings(fresh, task);
	if (task.status === "Done" && task.checklist.done < task.checklist.total) {
		warnings.push(
			`${task.id} is Done with ${task.checklist.total - task.checklist.done} unchecked items; ` +
				"the Definition of Done asks for every criterion checked with evidence",
		);
	}
	syncGenerated(fresh);
	return { task, warnings };
}

/** Checks or unchecks the `index`-th checkbox of a task body (0-based, code blocks skipped). */
export function setChecklistItem(board: Board, id: string, index: number, done: boolean): WriteResult {
	const current = getTask(board, id);
	const path = join(board.root, current.file);
	const text = readFileSync(path, "utf8");
	const match = FRONT_MATTER.exec(text);
	if (!match) throw new Error("file has no front matter");
	const head = text.slice(0, match[0].length);
	const lines = text.slice(match[0].length).split("\n");
	let seen = -1;
	let fenced = false;
	let hit = false;
	for (let i = 0; i < lines.length; i++) {
		if (FENCE.test(lines[i])) {
			fenced = !fenced;
			continue;
		}
		if (fenced) continue;
		const m = CHECKBOX.exec(lines[i]);
		if (!m || ++seen !== index) continue;
		lines[i] = `${m[1]}${done ? "x" : " "}${m[3]}${m[4]}`;
		hit = true;
		break;
	}
	if (!hit) throw new Error(`${current.id} has no checklist item ${index}`);
	const body = lines.join("\n");
	writeFileSync(path, setFrontMatter(head + body, [["last_updated", board.today]]));
	const fresh = loadBoard(board.root);
	syncGenerated(fresh);
	return { task: getTask(fresh, current.id), warnings: [] };
}

function slug(title: string): string {
	return title
		.normalize("NFD")
		.replace(/[̀-ͯ]/g, "")
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, "-")
		.replace(/^-|-$/g, "")
		.split("-")
		.slice(0, 6)
		.join("-");
}

/** Next free id for a prefix, keeping the digit width the prefix already uses (COV-05, DESK-097). */
export function nextId(board: Board, prefix: string): string {
	const p = prefix.toUpperCase();
	const nums = board.tasks
		.map((t) => /^([A-Z]+)-(\d+)$/.exec(t.id))
		.filter((m): m is RegExpExecArray => !!m && m[1] === p);
	const width = nums.length ? Math.max(...nums.map((m) => m[2].length)) : 3;
	const next = nums.length ? Math.max(...nums.map((m) => Number(m[2]))) + 1 : 1;
	return `${p}-${String(next).padStart(width, "0")}`;
}

export interface NewTask {
	prefix: string;
	title: string;
	area: Area;
	priority?: Priority;
	status?: Status;
	parent?: string;
	milestone: string;
	sprint?: string;
	tags?: string[];
	description?: string;
	steps?: string[];
	criteria?: string[];
}

export function createTask(board: Board, input: NewTask): WriteResult {
	if (!/^[A-Za-z]+$/.test(input.prefix)) throw new Error("prefix must be letters only, e.g. CORE");
	if (!AREAS.includes(input.area)) throw new Error(`area must be one of ${AREAS.join(", ")}`);
	validatePatch(board, input);
	const id = nextId(board, input.prefix);
	const fm: Record<string, string | string[]> = {
		id,
		title: input.title.trim(),
		status: input.status ?? "To Do",
		assignee: "bernardopg",
	};
	if (input.parent) fm.parent = input.parent;
	fm.priority = input.priority ?? "Medium";
	fm.milestone = input.milestone;
	if (input.sprint) fm.sprint = input.sprint;
	fm.tags = input.tags ?? [];
	fm.last_updated = board.today;
	const list = (items: string[] | undefined, fallback: string) =>
		(items?.length ? items : [fallback]).map((s) => `- [ ] ${s}`).join("\n");
	const body = [
		"## Description",
		"",
		input.description?.trim() || "TODO: what is wrong or missing, and why it matters.",
		"",
		"## Implementation Steps",
		"",
		list(input.steps, "TODO"),
		"",
		"## Acceptance Criteria",
		"",
		list(input.criteria, "TODO: a verifiable criterion"),
		"",
	].join("\n");
	const header = Object.entries(fm)
		.map(([k, v]) => `${k}: ${formatValue(v)}`)
		.join("\n");
	const path = join(tasksDir(board.root), input.area, `${id}-${slug(input.title)}.md`);
	writeFileSync(path, `---\n${header}\n---\n\n${body}`, { flag: "wx" });
	const fresh = loadBoard(board.root);
	syncGenerated(fresh);
	const task = getTask(fresh, id);
	return { task, warnings: wipWarnings(fresh, task) };
}

/** Adds a dated line at the top of the sprint's `## Log` section (creates the section if missing). */
export function addSprintLog(board: Board, sprintId: string, text: string): string {
	const sprint = board.sprints.find((s) => s.id === sprintId);
	if (!sprint) throw new Error(`sprint ${sprintId} not found`);
	const path = join(board.root, sprint.file);
	const content = readFileSync(path, "utf8");
	const line = `- ${board.today}: ${text.trim().replace(/\s*\n\s*/g, " ")}`;
	const heading = /^## Log[ \t]*\n+/m.exec(content);
	const next = heading
		? content.slice(0, heading.index + heading[0].length) + `${line}\n\n` + content.slice(heading.index + heading[0].length)
		: `${content.trimEnd()}\n\n## Log\n\n${line}\n`;
	writeFileSync(path, next);
	return line;
}

// ---------------------------------------------------------------------------------------------
// Generated blocks: tables in sprint files and ROADMAP.md rebuilt from front matter, so the
// status written by hand never drifts from the tasks again.

const NOTE = "_Generated from the task front matter by the board (`just board-sync`). Do not edit by hand._";
const STATUS_ORDER: Record<Status, number> = { "In Progress": 0, "To Do": 1, Done: 2 };
const PRIORITY_ORDER: Record<Priority, number> = { High: 0, Medium: 1, Low: 2 };

export function compareTasks(a: Task, b: Task): number {
	return (
		STATUS_ORDER[a.status] - STATUS_ORDER[b.status] ||
		PRIORITY_ORDER[a.priority] - PRIORITY_ORDER[b.priority] ||
		a.id.localeCompare(b.id, "en", { numeric: true })
	);
}

function cell(text: string): string {
	return text.replace(/\\/g, "\\\\").replace(/\|/g, "\\|").replace(/\n/g, " ");
}

function counts(tasks: Task[]) {
	const done = tasks.filter((t) => t.status === "Done").length;
	const doing = tasks.filter((t) => t.status === "In Progress").length;
	return { done, doing, todo: tasks.length - done - doing, total: tasks.length };
}

function pct(done: number, total: number): string {
	return total ? `${Math.round((done / total) * 100)}%` : "—";
}

function replaceBlock(text: string, name: string, content: string, insertAt: (text: string) => number): string {
	const begin = `<!-- board:begin ${name} -->`;
	const end = `<!-- board:end ${name} -->`;
	const block = `${begin}\n${content.trim()}\n${end}`;
	const from = text.indexOf(begin);
	const to = text.indexOf(end);
	if (from >= 0 && to > from) return text.slice(0, from) + block + text.slice(to + end.length);
	const at = insertAt(text);
	const before = text.slice(0, at).trimEnd();
	const after = text.slice(at).replace(/^\n+/, "");
	return `${before}\n\n${block}\n${after ? `\n${after}` : ""}`;
}

export function sprintBlock(board: Board, sprint: Sprint): string {
	const tasks = board.tasks.filter((t) => t.sprint === sprint.id).sort(compareTasks);
	const c = counts(tasks);
	const rows = tasks.map((t) => {
		const link = `[${t.id}](../${t.file.replace(/^\.tasks\//, "")})`;
		const check = t.checklist.total ? `${t.checklist.done}/${t.checklist.total}` : "—";
		return `| ${link} | ${cell(t.title)} | ${t.status} | ${t.priority} | ${check} |`;
	});
	return [
		"## Board",
		"",
		NOTE,
		"",
		`Progress: **${c.done}/${c.total} Done (${pct(c.done, c.total)})** · ${c.doing} In Progress · ${c.todo} To Do`,
		"",
		"| Task | Title | Status | Priority | Checklist |",
		"|---|---|---|---|---|",
		...rows,
	].join("\n");
}

export function roadmapBlock(board: Board): string {
	const milestones = board.milestones.map((m) => {
		const c = counts(board.tasks.filter((t) => t.milestone === m.id));
		return `| ${m.id} — ${cell(m.title)} | ${m.phase} | ${c.done} | ${c.doing} | ${c.todo} | ${pct(c.done, c.total)} |`;
	});
	const sprints = board.sprints.map((s) => {
		const c = counts(board.tasks.filter((t) => t.sprint === s.id));
		const name = s.id === board.currentSprint ? `**[${s.id}](sprints/${s.id}.md)** (current)` : `[${s.id}](sprints/${s.id}.md)`;
		const dates = s.start && s.end ? `${s.start} to ${s.end}` : "—";
		return `| ${name} | ${dates} | ${c.done}/${c.total} | ${c.doing} | ${pct(c.done, c.total)} |`;
	});
	return [
		"## Progress",
		"",
		NOTE,
		"",
		"| Milestone | Phase | Done | In Progress | To Do | Progress |",
		"|---|---|---|---|---|---|",
		...milestones,
		"",
		"| Sprint | Dates | Done | In Progress | Progress |",
		"|---|---|---|---|---|",
		...sprints,
	].join("\n");
}

function beforeHeading(pattern: RegExp): (text: string) => number {
	return (text) => {
		const m = pattern.exec(text);
		return m ? m.index : text.length;
	};
}

/** Rewrites the generated blocks; returns the files that changed. */
export function syncGenerated(board: Board): string[] {
	const changed: string[] = [];
	const write = (rel: string, update: (text: string) => string) => {
		const path = join(board.root, rel);
		const before = readFileSync(path, "utf8");
		const after = update(before);
		if (after !== before) {
			writeFileSync(path, after);
			changed.push(rel);
		}
	};
	for (const sprint of board.sprints) {
		write(sprint.file, (t) => replaceBlock(t, "board", sprintBlock(board, sprint), beforeHeading(/^## Log\b/m)));
	}
	if (existsSync(join(tasksDir(board.root), "ROADMAP.md"))) {
		write(".tasks/ROADMAP.md", (t) =>
			replaceBlock(t, "progress", roadmapBlock(board), beforeHeading(/^## (Done|Now|Next|Later) [—–-] M\d+/m)),
		);
	}
	return changed;
}

// ---------------------------------------------------------------------------------------------
// Integrity

export function checkBoard(board: Board): Issue[] {
	const issues: Issue[] = [];
	const ids = new Map<string, Task>();
	const sprintIds = new Set(board.sprints.map((s) => s.id));
	const milestoneIds = new Set(board.milestones.map((m) => m.id));
	for (const b of board.broken) issues.push({ level: "error", file: b.file, message: `unreadable front matter: ${b.message}` });
	for (const t of board.tasks) {
		const at = { id: t.id, file: t.file };
		if (ids.has(t.id)) issues.push({ level: "error", ...at, message: `duplicate id (also ${ids.get(t.id)!.file})` });
		ids.set(t.id, t);
	}
	for (const t of board.tasks) {
		const at = { id: t.id, file: t.file };
		if (!/^[A-Z]+-\d+$/.test(t.id)) issues.push({ level: "error", ...at, message: "id must look like ABC-123" });
		if (!basename(t.file).startsWith(`${t.id}-`) && basename(t.file) !== `${t.id}.md`) {
			issues.push({ level: "warn", ...at, message: "file name does not start with the id" });
		}
		if (t.rawStatus !== undefined) issues.push({ level: "error", ...at, message: `invalid status "${t.rawStatus}"` });
		if (t.rawPriority !== undefined) {
			issues.push({ level: "error", ...at, message: `invalid priority "${t.rawPriority}"` });
		}
		if (!t.assignee) issues.push({ level: "warn", ...at, message: "no assignee" });
		if (t.parent && !ids.has(t.parent)) issues.push({ level: "error", ...at, message: `parent ${t.parent} not found` });
		if (!t.milestone) issues.push({ level: "warn", ...at, message: "no milestone" });
		else if (milestoneIds.size && !milestoneIds.has(t.milestone)) {
			issues.push({ level: "error", ...at, message: `milestone ${t.milestone} is not in ROADMAP.md` });
		}
		if (t.sprint && !sprintIds.has(t.sprint)) issues.push({ level: "error", ...at, message: `sprint ${t.sprint} has no file` });
		if (t.status !== "Done" && !t.sprint) issues.push({ level: "warn", ...at, message: "open task without sprint" });
		if (t.status === "Done" && t.checklist.done < t.checklist.total) {
			issues.push({
				level: "warn",
				...at,
				message: `Done with ${t.checklist.total - t.checklist.done}/${t.checklist.total} items unchecked`,
			});
		}
		for (const dep of t.dependsOn) {
			if (!ids.has(dep)) issues.push({ level: "warn", ...at, message: `depends on unknown ${dep}` });
		}
	}
	if (board.currentSprint) {
		const wip = board.tasks.filter((t) => t.sprint === board.currentSprint && t.status === "In Progress" && !t.isEpic);
		if (wip.length > WIP_LIMIT) {
			issues.push({
				level: "warn",
				message: `${board.currentSprint} has ${wip.length} tasks In Progress (WIP limit ${WIP_LIMIT}): ${wip.map((t) => t.id).join(", ")}`,
			});
		}
	}
	return issues;
}

export function summarize(tasks: Task[]) {
	return counts(tasks);
}
