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
				// As in the shell, `[!` and `[^` negate, and a `]` right after them is a member.
				const negated = glob[i + 1] === '!' || glob[i + 1] === '^';
				const start = i + 1 + (negated ? 1 : 0);
				const end = glob.indexOf(']', glob[start] === ']' ? start + 1 : start);
				if (end > start) {
					const members = glob.slice(start, end).replace(/[\\\]^]/g, '\\$&');
					const characterClass = `[${negated ? '^' : ''}${members}]`;
					try {
						new RegExp(characterClass, 'i');
						out += characterClass;
					} catch {
						// Invalid ranges should match their literal spelling instead of breaking the filter.
						out += glob.slice(i, end + 1).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
					}
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
