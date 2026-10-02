import { expect, test } from "bun:test";
import { useUndoStore } from "./undoStore";

test("undo is library scoped, serial, and keeps failed entries for retry", async () => {
	useUndoStore.setState({entries: [], busy: false});
	let fail = true;
	useUndoStore.getState().record({libraryId: "a", label: "rename", run: async () => { if (fail) throw Error("conflict"); }});
	useUndoStore.getState().record({libraryId: "b", label: "other", run: async () => {}});
	await expect(useUndoStore.getState().undo("a")).rejects.toThrow("conflict");
	expect(useUndoStore.getState().entries).toHaveLength(2);
	expect(useUndoStore.getState().busy).toBe(false);
	fail = false;
	expect(await useUndoStore.getState().undo("a")).toBe("rename");
	expect(useUndoStore.getState().entries[0]?.libraryId).toBe("b");
});
