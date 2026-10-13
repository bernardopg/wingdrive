import { expect, test } from "bun:test";
import { planBatchRename } from "./batchRename";

const files = [
	{name: "IMG_001", extension: "jpg"},
	{name: "IMG_002", extension: "JPG"},
	{name: "notes", extension: null},
];

test("applies find/replace and numbering while keeping extensions", () => {
	const plan = planBatchRename(files, {find: "IMG_", replace: "trip-", template: "{n} {name}", start: 1, pad: 2});
	expect(plan.map((p) => p.newName)).toEqual(["01 trip-001.jpg", "02 trip-002.JPG", "03 notes"]);
	expect(plan.every((p) => p.error === null)).toBe(true);
});

test("flags empty, slashed and colliding names", () => {
	// Case-only differences collide on macOS and Windows file systems.
	const plan = planBatchRename(files, {find: "", replace: "", template: "same", start: 1, pad: 0});
	expect(plan.map((p) => p.error)).toEqual(["Duplicates another new name", "Duplicates another new name", null]);
	expect(planBatchRename(files.slice(0, 2), {find: "", replace: "", template: "x", start: 1, pad: 0})[0].error)
		.toBe("Duplicates another new name");
	expect(planBatchRename([files[2]], {find: "", replace: "", template: "a/b", start: 1, pad: 0})[0].error)
		.toBe("Name contains a slash");
	expect(planBatchRename([files[0]], {find: "", replace: "", template: " ", start: 1, pad: 0})[0].error)
		.toBe("Name is empty");
});
