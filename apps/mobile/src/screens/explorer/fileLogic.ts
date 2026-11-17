import type { FileSearchInput, WingPath } from "@wingdrive/ts-client";

/** The path of a file on this phone, or null when it lives on another device or in the cloud. */
export function localPath(path: WingPath, currentDeviceSlug: string | undefined): string | null {
	if (!currentDeviceSlug || typeof path !== "object" || !("Physical" in path)) return null;
	const physical = path.Physical;
	return physical.device_slug === currentDeviceSlug ? physical.path : null;
}

/** Whether `path` is a location root or inside one; such a folder cannot become a location. */
export function isInLocation(path: string, locationRoots: string[]): boolean {
	return locationRoots.some(
		(root) => path === root || path.startsWith(root.endsWith("/") ? root : `${root}/`),
	);
}

/** Library-wide lists opened from a wing item, the same queries as the desktop's views (MOB-017). */
export const LIBRARY_VIEWS = ["recents", "favorites"] as const;
export type LibraryView = (typeof LIBRARY_VIEWS)[number];

export function isLibraryView(view: string | undefined): view is LibraryView {
	return LIBRARY_VIEWS.includes(view as LibraryView);
}

export function libraryViewInput(view: LibraryView): FileSearchInput {
	return {
		query: "",
		scope: "Library",
		filters: {
			file_types: null,
			tags: null,
			date_range: null,
			size_range: null,
			locations: null,
			content_types: null,
			favorite: view === "favorites" ? true : null,
			include_hidden: null,
			include_archived: null,
			at_risk: null,
			on_volumes: null,
			not_on_volumes: null,
			min_volume_count: null,
			max_volume_count: null,
		},
		mode: "Fast",
		sort: {
			field: view === "recents" ? "IndexedAt" : "ModifiedAt",
			direction: "Desc",
		},
		pagination: { limit: view === "recents" ? 100 : 1000, offset: 0 },
	};
}

/** How long ago a date was, as a catalog key and its count, so the list can translate it. */
export function relativeDate(
	dateStr: string | null,
	now: Date = new Date(),
): { key: string; count?: number } | null {
	if (!dateStr) return null;
	const date = new Date(dateStr);
	if (Number.isNaN(date.getTime())) return null;
	const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));
	// A clock running behind on the other device can put a date in the future.
	if (diffDays <= 0) return { key: "Today" };
	if (diffDays === 1) return { key: "Yesterday" };
	if (diffDays < 7) return { key: "{{count}} days ago", count: diffDays };
	const [unit, count] =
		diffDays < 30
			? ["week", Math.floor(diffDays / 7)]
			: diffDays < 365
				? ["month", Math.floor(diffDays / 30)]
				: ["year", Math.floor(diffDays / 365)];
	return count === 1 ? { key: `1 ${unit} ago` } : { key: `{{count}} ${unit}s ago`, count };
}

const IMAGE_EXTENSIONS = new Set(["jpg", "jpeg", "png", "webp", "gif", "bmp", "heic", "heif", "avif"]);
const VIDEO_EXTENSIONS = new Set(["mp4", "m4v", "mov", "mkv", "webm", "3gp"]);

/** Whether the phone can draw a preview of a file itself: images, and a frame of videos. */
export function previewKind(extension: string | null | undefined): "image" | "video" | null {
	const ext = extension?.toLowerCase();
	if (!ext) return null;
	if (IMAGE_EXTENSIONS.has(ext)) return "image";
	if (VIDEO_EXTENSIONS.has(ext)) return "video";
	return null;
}
