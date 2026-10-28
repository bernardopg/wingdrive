/**
 * The absolute path a typed location means. A leading `~` is the home folder
 * when it is known (`~` alone, or `~/...`); any other path without a leading
 * slash is relative to the folder being shown, as in Dolphin and a shell.
 * `.` and `..` segments are resolved. Returns null for blank input or an
 * unresolved `~`.
 */
export function resolveTypedPath(
	input: string,
	home: string | null,
	current: string
): string | null {
	const typed = input.trim();
	if (!typed) return null;
	let joined: string;
	if (typed === '~' || typed.startsWith('~/')) {
		if (!home) return null;
		joined = `${home}/${typed.slice(1)}`;
	} else if (typed.startsWith('/')) {
		joined = typed;
	} else {
		joined = `${current}/${typed}`;
	}
	return normalizePath(joined);
}

function normalizePath(path: string): string {
	const segments: string[] = [];
	for (const segment of path.split('/')) {
		if (!segment || segment === '.') continue;
		if (segment === '..') segments.pop();
		else segments.push(segment);
	}
	return `/${segments.join('/')}`;
}
