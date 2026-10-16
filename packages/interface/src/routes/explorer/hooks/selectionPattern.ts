import {fileDisplayName, type File} from '@wingdrive/ts-client';

/**
 * Turns a shell glob into a case-insensitive whole-name matcher: `*` is any run,
 * `?` one character, `[abc]` a class. Several globs are separated by `;`
 * (`*.jpg; *.png`), since names often hold spaces and commas. Returns null when
 * nothing usable was typed.
 */
export function globMatcher(pattern: string): RegExp | null {
	const globs = pattern.split(';').map((glob) => glob.trim()).filter(Boolean);
	if (globs.length === 0) return null;
	const sources = globs.map((glob) => {
		let out = '';
		for (let i = 0; i < glob.length; i++) {
			const c = glob[i]!;
			if (c === '*') out += '.*';
			else if (c === '?') out += '.';
			else if (c === '[') {
				const end = glob.indexOf(']', i + 1);
				if (end > i + 1) {
					const body = glob.slice(i + 1, end).replace(/^!/, '^').replace(/\\/g, '\\\\');
					out += `[${body}]`;
					i = end;
				} else out += '\\[';
			} else out += c.replace(/[.+^${}()|\\\]]/g, '\\$&');
		}
		return out;
	});
	return new RegExp(`^(?:${sources.join('|')})$`, 'i');
}

/** The files whose full name (with extension) matches the pattern, in listing order. */
export function filesMatching(files: File[], pattern: string): File[] {
	const matcher = globMatcher(pattern);
	return matcher ? files.filter((file) => matcher.test(fileDisplayName(file))) : [];
}

/** Every listed file not in the current selection, in listing order. */
export function invertSelection(files: File[], selected: File[]): File[] {
	const ids = new Set(selected.map((file) => file.id));
	return files.filter((file) => !ids.has(file.id));
}
