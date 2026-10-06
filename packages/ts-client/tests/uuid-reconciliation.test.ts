import { describe, expect, test } from "bun:test";
import { QueryClient } from "@tanstack/react-query";
import { reconcileAlternateIds, updateSingleResource } from "../src/hooks/useNormalizedQuery";

describe("UUID reconciliation events", () => {
	const persistent = { id: "persistent", name: "photo.jpg", wing_path: { Physical: { path: "/photos/photo.jpg" } } };

	test("ResourceChanged updates an actual directory query cache", () => {
		const client = new QueryClient();
		const key = ["files.directory_listing", "/photos"];
		client.setQueryData(key, { files: [{ id: "temporary", name: "photo.jpg" }], total_count: 1 });
		updateSingleResource(persistent, { alternate_ids: ["temporary"], no_merge_fields: ["wing_path"] }, key, client);
		expect(client.getQueryData(key)).toEqual({ files: [persistent], total_count: 1 });
	});

	test("replaces a v4 entry in a wrapped directory listing", () => {
		const old = { files: [{ id: "temporary", name: "photo.jpg" }], total_count: 1 };
		const next = reconcileAlternateIds(old, persistent, ["temporary"], ["wing_path"]);
		expect(next.files).toEqual([persistent]);
		expect(next.total_count).toBe(1);
	});

	test("deduplicates when the persistent entry is already present", () => {
		const next = reconcileAlternateIds(
			[{ id: "temporary" }, persistent], persistent, ["temporary"], [],
		);
		expect(next).toHaveLength(1);
		expect(next[0].id).toBe("persistent");
	});

	test("updates a cached single-file query without touching unrelated entries", () => {
		const next = reconcileAlternateIds({ id: "temporary", name: "old" }, persistent, ["temporary"], []);
		expect(next.id).toBe("persistent");
		expect(reconcileAlternateIds({ id: "other" }, persistent, ["temporary"], [])).toEqual({ id: "other" });
	});
});
