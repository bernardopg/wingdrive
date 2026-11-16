import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import {
	checkBoard,
	checklistItems,
	createTask,
	loadBoard,
	nextId,
	setChecklistItem,
	setFrontMatter,
	syncGenerated,
	updateTask,
} from "../src/store";

let root: string;

const task = (fm: string, body = "## Steps\n\n- [ ] one\n- [x] two\n") => `---\n${fm}\n---\n\n${body}`;

beforeEach(() => {
	root = mkdtempSync(join(tmpdir(), "board-test-"));
	process.env.BOARD_TODAY = "2026-11-16";
	const t = join(root, ".tasks");
	for (const d of ["core", "interface", "sprints"]) mkdirSync(join(t, d), { recursive: true });
	writeFileSync(join(t, "PLANNING.md"), "# Planning\n");
	writeFileSync(
		join(t, "ROADMAP.md"),
		"# Roadmap\n\nIntro.\n\n## Done — M1: First\n\nText.\n\n## Now — M2: Second\n\nText.\n",
	);
	writeFileSync(
		join(t, "sprints", "S01.md"),
		"# Sprint S01 — 2026-11-10 a 2026-11-23\n\nMilestone: **M2**\n\n**Meta:** ship the\nboard.\n\n## Log\n\n- 2026-11-10: opened.\n",
	);
	writeFileSync(
		join(t, "core", "CORE-001-epic.md"),
		task("id: CORE-001\ntitle: 'Epic: core things'\nstatus: In Progress\nassignee: me\npriority: High\nmilestone: M2\nsprint: S01\ntags: [core, epic]"),
	);
	writeFileSync(
		join(t, "core", "CORE-002-child.md"),
		task(
			"id: CORE-002\ntitle: Child task here\nstatus: To Do\nassignee: me\nparent: CORE-001\npriority: Medium # keep me\nmilestone: M2\nsprint: S01\ntags: [core]\nrelated_tasks:\n  - CORE-001\nlast_updated: 2026-01-01",
			"## Steps\n\n```md\n- [ ] not a real item\n```\n\n- [ ] first\n  - [x] nested\n",
		),
	);
	writeFileSync(
		join(t, "interface", "DESK-009-broken.md"),
		task("id: DESK-009\ntitle: Broken parent\nstatus: Completed\nassignee: me\nparent: NOPE-000\npriority: Low\nmilestone: M1\ntags: []"),
	);
});

afterEach(() => rmSync(root, { recursive: true, force: true }));

describe("parsing", () => {
	test("reads tasks, sprints, milestones and the current sprint", () => {
		const b = loadBoard(root);
		expect(b.tasks.map((t) => t.id)).toEqual(["CORE-001", "CORE-002", "DESK-009"]);
		expect(b.milestones.map((m) => `${m.id}:${m.phase}`)).toEqual(["M1:Done", "M2:Now"]);
		expect(b.sprints[0]).toMatchObject({ id: "S01", start: "2026-11-10", end: "2026-11-23", goal: "ship the board." });
		expect(b.currentSprint).toBe("S01");
		const desk = b.tasks.find((t) => t.id === "DESK-009")!;
		expect(desk.status).toBe("Done");
		expect(b.tasks.find((t) => t.id === "CORE-001")!.isEpic).toBe(true);
	});

	test("checklist skips fenced code", () => {
		expect(checklistItems("```\n- [ ] no\n```\n- [ ] a\n- [x] b").map((i) => i.text)).toEqual(["a", "b"]);
	});
});

describe("writing", () => {
	test("setFrontMatter keeps other lines and comments", () => {
		const text = "---\nid: X-1\npriority: Medium # keep me\ntags:\n  - a\n  - b\n---\nbody\n";
		const out = setFrontMatter(text, [
			["tags", ["c", "d"]],
			["sprint", "S02"],
			["id", null],
		]);
		expect(out).toBe("---\npriority: Medium # keep me\ntags: [c, d]\nsprint: S02\n---\nbody\n");
	});

	test("updateTask changes only the touched keys and stamps last_updated", () => {
		const { task, warnings } = updateTask(loadBoard(root), "core-002", { status: "In Progress" });
		expect(task.status).toBe("In Progress");
		expect(task.lastUpdated).toBe("2026-11-16");
		expect(warnings).toEqual([]);
		const text = readFileSync(join(root, task.file), "utf8");
		expect(text).toContain("priority: Medium # keep me");
		expect(text).toContain("related_tasks:\n  - CORE-001");
	});

	test("updateTask rejects unknown sprint and warns on Done with open items", () => {
		expect(() => updateTask(loadBoard(root), "CORE-002", { sprint: "S99" })).toThrow(/S99/);
		const { warnings } = updateTask(loadBoard(root), "CORE-002", { status: "Done" });
		expect(warnings.join()).toMatch(/unchecked/);
	});

	test("clearing the sprint removes the key", () => {
		const { task } = updateTask(loadBoard(root), "CORE-002", { sprint: null });
		expect(task.sprint).toBeUndefined();
		expect(readFileSync(join(root, task.file), "utf8")).not.toContain("sprint:");
	});

	test("setChecklistItem toggles the right line, outside code fences", () => {
		const { task } = setChecklistItem(loadBoard(root), "CORE-002", 0, true);
		const text = readFileSync(join(root, task.file), "utf8");
		expect(text).toContain("- [ ] not a real item");
		expect(text).toContain("- [x] first");
		expect(task.checklist).toEqual({ done: 2, total: 2 });
		expect(() => setChecklistItem(loadBoard(root), "CORE-002", 5, true)).toThrow();
	});

	test("createTask picks the next id and writes a valid file", () => {
		const b = loadBoard(root);
		expect(nextId(b, "core")).toBe("CORE-003");
		expect(nextId(b, "NEW")).toBe("NEW-001");
		const { task } = createTask(b, {
			prefix: "CORE",
			title: "Ação nova: com vírgula, e acento",
			area: "core",
			milestone: "M2",
			sprint: "S01",
			parent: "CORE-001",
			criteria: ["it works"],
		});
		expect(task.id).toBe("CORE-003");
		expect(task.file).toBe(".tasks/core/CORE-003-acao-nova-com-virgula-e-acento.md");
		expect(task.title).toBe("Ação nova: com vírgula, e acento");
		expect(task.items.map((i) => i.text)).toEqual(["TODO", "it works"]);
	});
});

describe("generated blocks and checks", () => {
	test("sync writes the blocks once and is idempotent", () => {
		const b = loadBoard(root);
		expect(syncGenerated(b).sort()).toEqual([".tasks/ROADMAP.md", ".tasks/sprints/S01.md"]);
		expect(syncGenerated(loadBoard(root))).toEqual([]);
		const sprint = readFileSync(join(root, ".tasks/sprints/S01.md"), "utf8");
		expect(sprint.indexOf("<!-- board:begin quadro -->")).toBeLessThan(sprint.indexOf("## Log"));
		expect(sprint).toContain("Progresso: **0/2 Done (0%)**");
		const roadmap = readFileSync(join(root, ".tasks/ROADMAP.md"), "utf8");
		expect(roadmap.indexOf("board:begin progresso")).toBeLessThan(roadmap.indexOf("## Done — M1"));
		expect(roadmap).toContain("| M2 — Second | Now | 0 | 1 | 1 | 0% |");
	});

	test("check reports broken parent and open task without sprint", () => {
		updateTask(loadBoard(root), "CORE-002", { sprint: null });
		const issues = checkBoard(loadBoard(root));
		expect(issues).toContainEqual(expect.objectContaining({ level: "error", id: "DESK-009", message: "parent NOPE-000 not found" }));
		expect(issues).toContainEqual(expect.objectContaining({ level: "warn", id: "CORE-002", message: "open task without sprint" }));
	});

	test("a file with broken YAML is reported, not fatal", () => {
		writeFileSync(join(root, ".tasks/core/CORE-099-bad.md"), "---\nid: CORE-099\ntitle: Bad: yaml: here\n---\n");
		const b = loadBoard(root);
		expect(b.tasks).toHaveLength(3);
		expect(checkBoard(b)).toContainEqual(expect.objectContaining({ level: "error", file: ".tasks/core/CORE-099-bad.md" }));
	});
});
