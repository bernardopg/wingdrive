import { describe, expect, test } from "bun:test";
import type { WingPath } from "@wingdrive/ts-client";
import { useFileClipboard } from "../../stores/fileClipboard";
import {
	isInLocation,
	isLibraryView,
	libraryViewInput,
	localPath,
	pasteBlocker,
	previewKind,
	relativeDate,
	rowDisplayName,
	virtualTypeLabel,
} from "./fileLogic";

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

describe("previewKind", () => {
	test("recognizes images and videos in any case", () => {
		expect(previewKind("JPG")).toBe("image");
		expect(previewKind("heic")).toBe("image");
		expect(previewKind("mp4")).toBe("video");
	});

	test("has no preview for other files", () => {
		expect(previewKind("pdf")).toBeNull();
		expect(previewKind(null)).toBeNull();
		expect(previewKind("")).toBeNull();
	});
});

describe("isInLocation", () => {
	const roots = ["/storage/emulated/0/WingDriveTest"];

	test("covers the root and its subfolders", () => {
		expect(isInLocation("/storage/emulated/0/WingDriveTest", roots)).toBe(true);
		expect(isInLocation("/storage/emulated/0/WingDriveTest/Documentos", roots)).toBe(true);
	});

	test("leaves siblings that share a prefix and parents alone", () => {
		expect(isInLocation("/storage/emulated/0/WingDriveTest2", roots)).toBe(false);
		expect(isInLocation("/storage/emulated/0", roots)).toBe(false);
	});
});

describe("pasteBlocker", () => {
	const photo = { localPath: "/s/0/Fotos/a.jpg", isDirectory: false };
	const fotos = { localPath: "/s/0/Fotos", isDirectory: true };

	test("allows copies and moves to another folder", () => {
		expect(pasteBlocker("copy", [photo], "/s/0/Docs")).toBeNull();
		expect(pasteBlocker("move", [photo, fotos], "/s/0/Docs")).toBeNull();
	});

	test("allows a copy into the same folder, which gets a new name", () => {
		expect(pasteBlocker("copy", [photo], "/s/0/Fotos")).toBeNull();
	});

	test("blocks a move into the folder the item is in", () => {
		expect(pasteBlocker("move", [{ ...photo, localPath: "/a.jpg" }], "/")).toBe("The items are already in this folder");
		expect(pasteBlocker("move", [photo], "/s/0/Fotos")).toBe("The items are already in this folder");
		expect(pasteBlocker("move", [photo], "/s/0/Fotos/")).toBe("The items are already in this folder");
	});

	test("blocks a folder going into itself or a subfolder", () => {
		expect(pasteBlocker("copy", [fotos], "/s/0/Fotos")).toBe("A folder cannot go inside itself");
		expect(pasteBlocker("move", [fotos], "/s/0/Fotos/Sub")).toBe("A folder cannot go inside itself");
		expect(pasteBlocker("copy", [fotos], "/s/0/Fotos2")).toBeNull();
	});
});

describe("file clipboard", () => {
	const file = {
		path: physical("moto-g7", "/source/photo.jpg"),
		localPath: "/source/photo.jpg",
		name: "photo.jpg",
		isDirectory: false,
	};

	test("blocks duplicate submissions and allows retry after a dispatch failure", () => {
		const clipboard = useFileClipboard.getState();
		clipboard.set("move", [file]);
		expect(clipboard.beginPaste("/source")).toBeNull();
		const first = clipboard.beginPaste("/destination")!;
		expect(first).toEqual({ mode: "move", paths: [file.path] });
		expect(clipboard.beginPaste("/destination")).toBeNull();
		clipboard.finishPaste(first.paths, false);
		expect(useFileClipboard.getState().items).toEqual([file]);
		const retry = clipboard.beginPaste("/destination")!;
		clipboard.finishPaste(retry.paths, true);
		expect(useFileClipboard.getState().items).toEqual([]);
	});

	test("an older dispatch cannot clear a newer selection", () => {
		const clipboard = useFileClipboard.getState();
		clipboard.set("move", [file]);
		const old = clipboard.beginPaste("/destination")!;
		clipboard.set("copy", [file]);
		const current = clipboard.beginPaste("/destination")!;
		clipboard.finishPaste(old.paths, true);
		expect(useFileClipboard.getState().pendingPaths).toBe(current.paths);
		expect(useFileClipboard.getState().mode).toBe("copy");
		clipboard.finishPaste(current.paths, true);
	});
});

describe("virtual rows (MOB-017)", () => {
	const catalog: Record<string, string> = { "Internal storage": "Armazenamento interno" };
	const translate = (key: string) => catalog[key] ?? null;

	test("the kind of a row is a catalog key, not the raw type", () => {
		expect(virtualTypeLabel("location")).toBe("Location");
		expect(virtualTypeLabel("volume")).toBe("Volume");
		expect(virtualTypeLabel("device")).toBe("Device");
		expect(virtualTypeLabel("other")).toBeNull();
		expect(virtualTypeLabel(undefined)).toBeNull();
	});

	test("system volume names are translated, user names are not", () => {
		expect(rowDisplayName("Internal storage", "volume", translate)).toBe("Armazenamento interno");
		expect(rowDisplayName("home", "volume", translate)).toBe("home");
		expect(rowDisplayName("Internal storage", "location", translate)).toBe("Internal storage");
		expect(rowDisplayName("Internal storage", undefined, translate)).toBe("Internal storage");
	});
});
