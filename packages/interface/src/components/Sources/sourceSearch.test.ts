import { describe, expect, test } from "bun:test";
import type { SourceSearchHit } from "@wingdrive/ts-client";
import { groupHitsBySource, hasSearchableWord, sourceItemLink, splitSnippet } from "./sourceSearch";

function hit(id: string, sourceId: string, sourceName = sourceId): SourceSearchHit {
	return {
		id,
		external_id: `ext-${id}`,
		title: `Title ${id}`,
		subtitle: null,
		preview: null,
		snippet: null,
		date: null,
		source_id: sourceId,
		source_name: sourceName,
		adapter_id: "obsidian",
		data_type: "note",
		flagged: false,
	};
}

describe("splitSnippet", () => {
	test("marks the matched terms and drops the markers", () => {
		expect(splitSnippet("…plan the \u0002harness\u0003 for \u0002harness\u0003 work…")).toEqual([
			{ text: "…plan the ", match: false },
			{ text: "harness", match: true },
			{ text: " for ", match: false },
			{ text: "harness", match: true },
			{ text: " work…", match: false },
		]);
	});

	test("keeps text without markers whole", () => {
		expect(splitSnippet("plain")).toEqual([{ text: "plain", match: false }]);
		expect(splitSnippet("")).toEqual([]);
	});
});

describe("groupHitsBySource", () => {
	test("keeps rank order, caps each source and counts the rest", () => {
		const groups = groupHitsBySource(
			[hit("1", "b", "Issues"), hit("2", "a", "Vault"), hit("3", "b"), hit("4", "b")],
			2,
		);
		expect(groups.map((g) => g.sourceId)).toEqual(["b", "a"]);
		expect(groups[0].sourceName).toBe("Issues");
		expect(groups[0].hits.map((h) => h.id)).toEqual(["1", "3"]);
		expect(groups[0].hidden).toBe(1);
		expect(groups[1].hits.map((h) => h.id)).toEqual(["2"]);
		expect(groups[1].hidden).toBe(0);
	});

	test("an empty search has no groups", () => {
		expect(groupHitsBySource([], 5)).toEqual([]);
	});
});

describe("sourceItemLink", () => {
	test("opens the source searched, with the item selected", () => {
		expect(sourceItemLink("s 1", "weekly review", "item/2")).toBe(
			"/sources/s%201?q=weekly+review&item=item%2F2",
		);
		expect(sourceItemLink("s1", "  ")).toBe("/sources/s1");
	});
});

describe("hasSearchableWord", () => {
	test("needs a letter or a digit", () => {
		expect(hasSearchableWord("ação")).toBe(true);
		expect(hasSearchableWord("42")).toBe(true);
		expect(hasSearchableWord(" -*\"() ")).toBe(false);
	});
});
