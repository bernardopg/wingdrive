import type {File} from '@wingdrive/ts-client';

/** Absolute path of a local entry, or `null` for virtual and content paths. */
export function physicalPathOf(file: File): string | null {
	return 'Physical' in file.wing_path ? file.wing_path.Physical.path : null;
}

/**
 * Expanded folders after toggling `path`. Collapsing a folder also collapses
 * the folders inside it, so their listings stop loading in the background.
 */
export function toggleExpandedPath(expanded: string[], path: string): string[] {
	if (!expanded.includes(path)) return [...expanded, path];
	return expanded.filter((entry) => entry !== path && !entry.startsWith(`${path}/`));
}

/** Expanded folders that sit inside `folder`, the one the list shows. */
export function expandedWithin(expanded: string[], folder: string | null): string[] {
	if (!folder) return [];
	const prefix = folder.endsWith('/') ? folder : `${folder}/`;
	return expanded.filter((path) => path.startsWith(prefix));
}

/**
 * Children of each expanded folder keyed by the folder's row id, and the
 * table's expanded state, from listings that arrive in `paths` order.
 *
 * A listing is only attached once its folder is on screen, which is why
 * folders are matched level by level: a nested folder's row only exists after
 * its parent's listing has arrived.
 */
export function attachChildren(
	files: File[],
	paths: string[],
	listings: (File[] | undefined)[]
): {subRows: Map<string, File[]>; expanded: Record<string, boolean>} {
	const listingByPath = new Map<string, File[]>();
	paths.forEach((path, index) => {
		const listing = listings[index];
		if (listing) listingByPath.set(path, listing);
	});

	const subRows = new Map<string, File[]>();
	const expanded: Record<string, boolean> = {};
	const visit = (level: File[]) => {
		for (const file of level) {
			const path = physicalPathOf(file);
			if (file.kind !== 'Directory' || !path || !paths.includes(path)) continue;
			expanded[file.id] = true;
			const children = listingByPath.get(path);
			if (children) {
				subRows.set(file.id, children);
				visit(children);
			}
		}
	};
	visit(files);
	return {subRows, expanded};
}

/** Each top-level row followed by its expanded descendants, keyed by the top row's id. */
export function blocksByTopLevel<R extends {id: string; depth: number}>(rows: R[]): Map<string, R[]> {
	const blocks = new Map<string, R[]>();
	let current: R[] | null = null;
	for (const row of rows) {
		if (row.depth === 0 || !current) {
			current = [row];
			blocks.set(row.id, current);
		} else {
			current.push(row);
		}
	}
	return blocks;
}
