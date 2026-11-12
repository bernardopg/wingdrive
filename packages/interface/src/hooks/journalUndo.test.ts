import { expect, test } from "bun:test";
import type { JobOutput } from "@wingdrive/ts-client";
import { journaledUndo } from "./journalUndo";
import type { JobResult } from "./useWaitForJob";
import { useUndoStore } from "./undoStore";

const messages = {
	skipped: (count: number, reason: string) => `${count} skipped: ${reason}`,
	cancelled: () => "cancelled",
};

function undone(skipped: string[] = []): JobResult {
	const output: JobOutput = {
		type: "FileUndo",
		data: { undone_count: 1, skipped_count: skipped.length, skipped },
	};
	return { status: "completed", output };
}

test("several operations are undone newest first", async () => {
	const calls: string[] = [];
	const run = journaledUndo(["rename-1", "rename-2", "rename-3"], async (id) => {
		calls.push(id);
		return undone();
	}, messages);
	await run();
	expect(calls).toEqual(["rename-3", "rename-2", "rename-1"]);
});

test("a retry after a failure only undoes what is left", async () => {
	const calls: string[] = [];
	let failOn: string | null = "first";
	const run = journaledUndo(["first", "second"], async (id) => {
		calls.push(id);
		if (id === failOn) return { status: "failed", error: "it is no longer there" };
		return undone();
	}, messages);

	await expect(run()).rejects.toThrow("it is no longer there");
	expect(calls).toEqual(["second", "first"]);

	failOn = null;
	await run();
	// The second operation is never sent to the core twice.
	expect(calls).toEqual(["second", "first", "first"]);
});

test("skipped items and cancellation are reported, and the step stays", async () => {
	let result: JobResult = undone(["/tmp/a: it was changed after the operation"]);
	const run = journaledUndo(["job"], async () => result, messages);
	await expect(run()).rejects.toThrow("1 skipped: /tmp/a: it was changed after the operation");

	result = { status: "cancelled" };
	await expect(run()).rejects.toThrow("cancelled");

	// A timeout leaves the job running on its own; the step is done.
	result = { status: "timeout" };
	await run();
	let calls = 0;
	result = undone();
	await journaledUndo([], async () => { calls += 1; return result; }, messages)();
	expect(calls).toBe(0);
});

test("the store keeps a journaled step that failed and drops it once undone", async () => {
	useUndoStore.setState({ entries: [], busy: false });
	let answer: JobResult = { status: "failed", error: "taken again" };
	useUndoStore.getState().record({
		libraryId: "lib",
		label: "new folder",
		run: journaledUndo(["create"], async () => answer, messages),
	});
	await expect(useUndoStore.getState().undo("lib")).rejects.toThrow("taken again");
	expect(useUndoStore.getState().entries).toHaveLength(1);

	answer = undone();
	expect(await useUndoStore.getState().undo("lib")).toBe("new folder");
	expect(useUndoStore.getState().entries).toHaveLength(0);
});
