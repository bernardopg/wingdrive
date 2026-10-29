/**
 * Path helpers for the sidebar folder tree. Paths are absolute POSIX paths
 * without a trailing slash, except the filesystem root `/`.
 */

export function normalizeFolder(path: string): string {
	const trimmed = path.replace(/\/+$/, '');
	return trimmed === '' ? '/' : trimmed;
}

/** Whether `path` is `root` or inside it. */
export function isWithin(path: string, root: string): boolean {
	const p = normalizeFolder(path);
	const r = normalizeFolder(root);
	return r === '/' ? p.startsWith('/') : p === r || p.startsWith(`${r}/`);
}

/**
 * Folders to expand so `path` is visible under `root`: `root` and every
 * folder between it and `path`, not `path` itself. Empty when `path` is not
 * inside `root`.
 */
export function ancestorsWithin(path: string, root: string): string[] {
	const p = normalizeFolder(path);
	const r = normalizeFolder(root);
	if (p === r || !isWithin(p, r)) return [];

	const rest = (r === '/' ? p.slice(1) : p.slice(r.length + 1)).split('/');
	const ancestors = [r];
	let current = r;
	for (const name of rest.slice(0, -1)) {
		current = current === '/' ? `/${name}` : `${current}/${name}`;
		ancestors.push(current);
	}
	return ancestors;
}

/**
 * The root a path belongs to, preferring the deepest one, so a folder in the
 * home folder follows the home tree rather than the filesystem root.
 */
export function rootFor(path: string, roots: string[]): string | null {
	let best: string | null = null;
	for (const root of roots) {
		if (isWithin(path, root) && (!best || normalizeFolder(root).length > best.length)) {
			best = normalizeFolder(root);
		}
	}
	return best;
}
