#!/usr/bin/env bun
// Entry point: `serve` (web board), `mcp` (stdio server for agents), `sync` (regenerate tables),
// `check` (integrity report; exits 1 on errors).

import { checkBoard, findRoot, loadBoard, syncGenerated } from "./store";

const [command = "serve", ...rest] = process.argv.slice(2);

function flag(name: string): string | undefined {
	const i = rest.indexOf(`--${name}`);
	return i >= 0 ? (rest[i + 1] ?? "") : undefined;
}

switch (command) {
	case "serve": {
		const { serve } = await import("./server");
		serve({
			port: Number(flag("port") ?? process.env.BOARD_PORT ?? 7357),
			open: rest.includes("--open"),
			dev: rest.includes("--dev"),
		});
		break;
	}
	case "mcp": {
		const { runMcp } = await import("./mcp");
		await runMcp();
		break;
	}
	case "sync": {
		const changed = syncGenerated(loadBoard(findRoot()));
		console.log(changed.length ? `regenerated ${changed.join(", ")}` : "generated tables already up to date");
		break;
	}
	case "check": {
		const board = loadBoard(findRoot());
		const issues = checkBoard(board);
		const generated = syncGenerated(board);
		const strict = rest.includes("--strict");
		for (const i of issues) console.log(`${i.level.padEnd(5)} ${i.id ?? ""} ${i.message}`.replace(/ {2,}/g, " "));
		const errors = issues.filter((i) => i.level === "error").length;
		console.log(
			`${board.tasks.length} tasks, ${errors} errors, ${issues.length - errors} warnings` +
				(generated.length ? `; regenerated ${generated.join(", ")}` : ""),
		);
		process.exit(errors || (strict && generated.length) ? 1 : 0);
	}
	default:
		console.error("usage: board <serve [--port N] [--open] [--dev] | mcp | sync | check [--strict]>");
		process.exit(2);
}
