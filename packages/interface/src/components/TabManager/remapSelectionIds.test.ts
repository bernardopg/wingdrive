import { describe, expect, test } from "bun:test";
import { remapSelectionIds } from "./remapSelectionIds";

describe("UUID reconciliation across tabs", () => {
	test("replaces selected IDs in active and inactive tabs without losing other selections", () => {
		const selections = new Map([
			["active", ["temporary", "other"]],
			["background", ["temporary", "persistent"]],
			["unrelated", ["other"]],
		]);
		const updated = remapSelectionIds(selections, ["temporary"], "persistent");
		expect(updated.get("active")).toEqual(["persistent", "other"]);
		expect(updated.get("background")).toEqual(["persistent"]);
		expect(updated.get("unrelated")).toEqual(["other"]);
		expect(selections.get("active")).toEqual(["temporary", "other"]);
	});

	test("leaves state untouched when event is unrelated", () => {
		const selections = new Map([["tab", ["other"]]]);
		expect(remapSelectionIds(selections, ["temporary"], "persistent")).toBe(selections);
	});
});
