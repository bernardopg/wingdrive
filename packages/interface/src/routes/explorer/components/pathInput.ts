/**
 * The absolute path a typed location means. A leading `~` is the home folder
 * when it is known (`~` alone, or `~/...`); any other path without a leading
 * slash is read from the root. Returns null for blank input or an unresolved `~`.
 */
export function resolveTypedPath(input: string, home: string | null): string | null {
	const typed = input.trim();
	if (!typed) return null;
	if (typed === '~' || typed.startsWith('~/')) {
		if (!home) return null;
		const root = home.replace(/\/+$/, '') || '/';
		const rest = typed.slice(2);
		return rest ? `${root === '/' ? '' : root}/${rest}` : root;
	}
	return typed.startsWith('/') ? typed : `/${typed}`;
}
