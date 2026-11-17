// Local web board. Binds to 127.0.0.1 only and refuses cross-origin writes, so a web page open in
// the browser cannot change tasks through it. Watches `.tasks/` and pushes changes over SSE, so
// edits from agents (MCP), editors or git show up without a reload.

import { watch } from "node:fs";
import { join } from "node:path";

import index from "../ui/index.html";
import {
	type Board,
	checkBoard,
	createTask,
	findRoot,
	getTask,
	loadBoard,
	type NewTask,
	setChecklistItem,
	syncGenerated,
	type TaskPatch,
	updateTask,
} from "./store";

interface Options {
	port: number;
	open: boolean;
	dev: boolean;
}

export function serve({ port, open, dev }: Options): void {
	const root = findRoot();
	let board: Board = loadBoard(root);
	const clients = new Set<ReadableStreamDefaultController<string>>();

	const broadcast = (event: string, data: unknown) => {
		const frame = `event: ${event}\ndata: ${JSON.stringify(data)}\n\n`;
		for (const c of clients) {
			try {
				c.enqueue(frame);
			} catch {
				clients.delete(c);
			}
		}
	};

	const reload = () => {
		const before = new Map(board.tasks.map((t) => [t.id, `${t.status}|${t.lastUpdated}|${t.checklist.done}|${t.sprint}|${t.title}`]));
		board = loadBoard(root);
		const changed = board.tasks
			.filter((t) => before.get(t.id) !== `${t.status}|${t.lastUpdated}|${t.checklist.done}|${t.sprint}|${t.title}`)
			.map((t) => t.id);
		broadcast("board", { changed });
	};

	let timer: ReturnType<typeof setTimeout> | undefined;
	watch(join(root, ".tasks"), { recursive: true }, () => {
		clearTimeout(timer);
		timer = setTimeout(reload, 150);
	});

	const snapshot = () => ({ ...board, issues: checkBoard(board) });

	const sameOrigin = (req: Request) => {
		const origin = req.headers.get("origin");
		const host = req.headers.get("host");
		return !origin || (host !== null && new URL(origin).host === host);
	};

	const guard = async <T>(req: Request, run: (body: T) => unknown) => {
		if (!sameOrigin(req) || !req.headers.get("content-type")?.includes("application/json")) {
			return Response.json({ error: "cross-origin or non-JSON write refused" }, { status: 403 });
		}
		try {
			const result = run((await req.json()) as T);
			board = loadBoard(root);
			return Response.json(result);
		} catch (e) {
			return Response.json({ error: (e as Error).message }, { status: 400 });
		}
	};

	const server = Bun.serve({
		hostname: "127.0.0.1",
		port,
		idleTimeout: 0,
		development: dev ? { hmr: true, console: true } : false,
		routes: {
			"/": index,
			"/api/board": { GET: () => Response.json(snapshot()) },
			"/api/tasks": { POST: (req) => guard<NewTask>(req, (body) => createTask(board, body)) },
			"/api/tasks/:id": {
				GET: (req) => {
					try {
						return Response.json(getTask(board, req.params.id));
					} catch (e) {
						return Response.json({ error: (e as Error).message }, { status: 404 });
					}
				},
				PATCH: (req) => guard<TaskPatch>(req, (patch) => updateTask(board, req.params.id, patch)),
			},
			"/api/tasks/:id/checklist": {
				POST: (req) =>
					guard<{ index: number; done: boolean }>(req, ({ index, done }) =>
						setChecklistItem(board, req.params.id, index, done),
					),
			},
			"/api/sync": { POST: (req) => guard<object>(req, () => ({ changed: syncGenerated(board) })) },
			"/api/events": {
				GET: (req) => {
					let ctrl: ReadableStreamDefaultController<string>;
					let beat: ReturnType<typeof setInterval>;
					const stream = new ReadableStream<string>({
						start(c) {
							ctrl = c;
							clients.add(c);
							c.enqueue("retry: 2000\n\n");
							beat = setInterval(() => c.enqueue(": ping\n\n"), 20_000);
						},
						cancel() {
							clearInterval(beat);
							clients.delete(ctrl);
						},
					});
					req.signal.addEventListener("abort", () => {
						clearInterval(beat);
						clients.delete(ctrl);
					});
					return new Response(stream, {
						headers: { "content-type": "text/event-stream", "cache-control": "no-cache", connection: "keep-alive" },
					});
				},
			},
		},
		fetch: () => new Response("not found", { status: 404 }),
	});

	const url = `http://${server.hostname}:${server.port}`;
	console.log(`WingDrive board on ${url} (tasks in ${join(root, ".tasks")})`);
	if (open) Bun.spawn(["xdg-open", url], { stdout: "ignore", stderr: "ignore" });
}
