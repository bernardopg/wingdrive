import { describe, expect, test } from "bun:test";
import type { WingPath } from "@wingdrive/ts-client";
import { isLibraryView, libraryViewInput, localPath, relativeDate } from "./fileLogic";

const physical = (device_slug: string, path: string) =>
	({ Physical: { device_slug, path } }) as WingPath;

describe("localPath", () => {
	test("returns the path of a file on this phone", () => {
		expect(localPath(physical("moto-g7", "/storage/emulated/0/a.txt"), "moto-g7")).toBe(
			"/storage/emulated/0/a.txt",
		);
	});

	test("rejects files on another device", () => {
		expect(localPath(physical("arch-desktop", "/home/u/a.txt"), "moto-g7")).toBeNull();
	});

	test("rejects every path while the current device is unknown", () => {
		expect(localPath(physical("moto-g7", "/a"), undefined)).toBeNull();
	});

	test("rejects cloud and content paths", () => {
		expect(localPath({ Content: { content_id: "x" } } as unknown as WingPath, "moto-g7")).toBeNull();
		expect(localPath({ Cloud: {} } as unknown as WingPath, "moto-g7")).toBeNull();
	});
});

describe("library views", () => {
	test("only recents and favorites are library views", () => {
		expect(isLibraryView("recents")).toBe(true);
		expect(isLibraryView("favorites")).toBe(true);
		expect(isLibraryView("device")).toBe(false);
		expect(isLibraryView(undefined)).toBe(false);
	});

	test("favorites filter on the favorite flag", () => {
		const input = libraryViewInput("favorites");
		expect(input.filters.favorite).toBe(true);
		expect(input.scope).toBe("Library");
	});

	test("recents sort by index time, newest first, without a favorite filter", () => {
		const input = libraryViewInput("recents");
		expect(input.filters.favorite).toBeNull();
		expect(input.sort).toEqual({ field: "IndexedAt", direction: "Desc" });
		expect(input.query).toBe("");
	});
});

describe("relativeDate", () => {
	const now = new Date("2026-11-17T12:00:00Z");
	const daysAgo = (days: number) => new Date(now.getTime() - days * 86_400_000).toISOString();

	test("names today and yesterday", () => {
		expect(relativeDate(daysAgo(0), now)).toEqual({ key: "Today" });
		expect(relativeDate(daysAgo(1), now)).toEqual({ key: "Yesterday" });
	});

	test("counts days, weeks, months and years", () => {
		expect(relativeDate(daysAgo(3), now)).toEqual({ key: "{{count}} days ago", count: 3 });
		expect(relativeDate(daysAgo(14), now)).toEqual({ key: "{{count}} weeks ago", count: 2 });
		expect(relativeDate(daysAgo(90), now)).toEqual({ key: "{{count}} months ago", count: 3 });
		expect(relativeDate(daysAgo(800), now)).toEqual({ key: "{{count}} years ago", count: 2 });
	});

	test("uses the singular for one week, month or year", () => {
		expect(relativeDate(daysAgo(8), now)).toEqual({ key: "1 week ago" });
		expect(relativeDate(daysAgo(45), now)).toEqual({ key: "1 month ago" });
		expect(relativeDate(daysAgo(400), now)).toEqual({ key: "1 year ago" });
	});

	test("treats future dates as today and ignores missing or broken dates", () => {
		expect(relativeDate(daysAgo(-2), now)).toEqual({ key: "Today" });
		expect(relativeDate(null, now)).toBeNull();
		expect(relativeDate("not a date", now)).toBeNull();
	});
});
