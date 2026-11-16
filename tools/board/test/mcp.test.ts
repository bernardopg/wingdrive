import { afterAll, beforeAll, expect, test } from "bun:test";
import { cpSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";

// Runs the real stdio server against a copy of the repository's .tasks/.
let root: string;
let client: Client;

const call = async (name: string, args: Record<string, unknown> = {}) => {
	const res = (await client.callTool({ name, arguments: args })) as { content: { text: string }[]; isError?: boolean };
	return { text: res.content[0].text, isError: !!res.isError };
};

beforeAll(async () => {
	root = mkdtempSync(join(tmpdir(), "board-mcp-"));
	cpSync(join(import.meta.dir, "../../../.tasks"), join(root, ".tasks"), { recursive: true });
	client = new Client({ name: "test", version: "0" });
	await client.connect(
		new StdioClientTransport({
			command: process.execPath,
			args: [join(import.meta.dir, "../src/cli.ts"), "mcp"],
			env: { ...process.env, BOARD_ROOT: root, BOARD_TODAY: "2026-11-16" } as Record<string, string>,
		}),
	);
});

afterAll(async () => {
	await client.close();
	rmSync(root, { recursive: true, force: true });
});

test("lists the tools", async () => {
	const { tools } = await client.listTools();
	expect(tools.map((t) => t.name).sort()).toEqual(
		["check", "create_task", "get_task", "list_tasks", "overview", "set_checklist_item", "sprint_log", "update_task"].sort(),
	);
});

test("overview names the current sprint", async () => {
	const { text } = await call("overview");
	expect(text).toContain("## Current sprint S05");
});

test("moves a task and logs it in the sprint", async () => {
	const todo = await call("list_tasks", { sprint: "S06", status: "To Do", limit: 1 });
	const id = /- ([A-Z]+-\d+)/.exec(todo.text)![1];
	const moved = await call("update_task", { id, status: "In Progress" });
	expect(moved.isError).toBe(false);
	expect(moved.text).toContain(`${id} (In Progress`);
	const detail = await call("get_task", { id });
	expect(detail.text).toContain('"status": "In Progress"');
	expect(detail.text).toContain('"last_updated": "2026-11-16"');
	const log = await call("sprint_log", { text: `${id} started by an agent`, sprint: "S06" });
	expect(log.text).toContain(`- 2026-11-16: ${id} started by an agent`);
	expect(readFileSync(join(root, ".tasks/sprints/S06.md"), "utf8")).toContain(`${id} started by an agent`);
});

test("rejects bad input with an error result", async () => {
	const res = await call("update_task", { id: "NOPE-1", status: "Done" });
	expect(res.isError).toBe(true);
	expect(res.text).toContain("not found");
});
