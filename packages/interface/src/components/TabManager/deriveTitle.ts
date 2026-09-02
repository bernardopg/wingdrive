/**
 * # Tab titles
 *
 * `deriveTitleFromPath` turns a router location into the label shown on a tab.
 * It lives in its own module because both the tab store (when creating a tab)
 * and the navigation syncer (when the route changes) need the exact same
 * mapping; two copies previously drifted apart and produced different titles
 * for the same route.
 *
 * Returning `null` means "leave the current title alone": some routes set their
 * own title once the underlying resource loads.
 */

const ROUTE_TITLES: Record<string, string> = {
	"/": "Overview",
	"/favorites": "Favorites",
	"/recents": "Recents",
	"/file-kinds": "File Kinds",
	"/sources": "Sources",
	"/sources/adapters": "Adapters",
	"/search": "Search",
	"/jobs": "Jobs",
	"/daemon": "Daemon",
};

export function deriveTitleFromPath(
	pathname: string,
	search: string,
): string | null {
	if (ROUTE_TITLES[pathname]) {
		return ROUTE_TITLES[pathname];
	}

	if (pathname.startsWith("/tag/")) {
		const tagId = pathname.split("/")[2];
		return tagId ? `Tag: ${tagId.slice(0, 8)}...` : "Tag";
	}

	// Title is set by the SourceDetail component once the source resolves.
	if (pathname.startsWith("/sources/") && pathname !== "/sources/adapters") {
		return null;
	}

	if (pathname === "/explorer" && search) {
		const params = new URLSearchParams(search);

		if (params.get("view") === "device") {
			return "This Device";
		}

		const pathParam = params.get("path");
		if (pathParam) {
			try {
				const sdPath = JSON.parse(decodeURIComponent(pathParam));
				if (sdPath?.Physical?.path) {
					const fullPath = sdPath.Physical.path as string;
					const parts = fullPath.split("/").filter(Boolean);
					return parts[parts.length - 1] || "Explorer";
				}
			} catch {
				// Malformed path param: fall back to the generic label.
			}
		}
		return "Explorer";
	}

		return "WingDrive";
}
