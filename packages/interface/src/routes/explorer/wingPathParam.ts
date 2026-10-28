import type {WingPath} from '@wingdrive/ts-client';

/**
 * Reads the WingPath in an Explorer URL's `path` parameter, as returned by
 * `URLSearchParams.get`, which already undid the encoding. Decoding it a second
 * time threw on folder names holding `%` (such as `100%`); the second decode
 * is kept only as a fallback for links encoded twice.
 */
export function wingPathFromParam(raw: string): WingPath | null {
	for (const text of [raw, safeDecode(raw)]) {
		if (text === null) continue;
		try {
			const parsed = JSON.parse(text) as unknown;
			if (parsed && typeof parsed === 'object') return parsed as WingPath;
		} catch {
			// Try the next form.
		}
	}
	return null;
}

function safeDecode(text: string): string | null {
	try {
		return decodeURIComponent(text);
	} catch {
		return null;
	}
}

/** The folder a tab shows, from its saved route, or null when it shows something else. */
export function folderOfRoute(route: string): WingPath | null {
	const [pathname, search = ''] = route.split('?');
	if (pathname !== '/explorer') return null;
	const raw = new URLSearchParams(search).get('path');
	const path = raw ? wingPathFromParam(raw) : null;
	return path && ('Physical' in path || 'Cloud' in path) ? path : null;
}
