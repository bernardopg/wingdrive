import { expect, test } from "bun:test";
import type { CompareEntry } from "@wingdrive/ts-client";
import { mergeDirection } from "./CompareFoldersModal";

const file = (modified: number) => ({ is_dir: false, size: 1, modified });

test("one-sided entries copy toward the side that lacks them, changed files toward the older side", () => {
	const entry = (e: Partial<CompareEntry>) => ({ relative_path: "x", left: null, right: null, ...e }) as CompareEntry;
	expect(mergeDirection(entry({ status: "OnlyLeft", left: file(1) }))).toBe("toRight");
	expect(mergeDirection(entry({ status: "OnlyRight", right: file(1) }))).toBe("toLeft");
	expect(mergeDirection(entry({ status: "Different", left: file(5), right: file(9) }))).toBe("toLeft");
	expect(mergeDirection(entry({ status: "Different", left: file(9), right: file(5) }))).toBe("toRight");
	expect(mergeDirection(entry({ status: "Different", left: file(9), right: { is_dir: true, size: 0, modified: 1 } }))).toBeNull();
});
