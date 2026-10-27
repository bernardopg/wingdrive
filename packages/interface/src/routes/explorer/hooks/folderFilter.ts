import {fileDisplayName, type File} from '@wingdrive/ts-client';
import {globMatcher} from './selectionPattern';

const GLOB_CHARS = /[*?[]/;

/**
 * The files whose name contains the filter text, case-insensitive. Text holding
 * `*`, `?` or `[` is read as a glob over the whole name instead, the same syntax
 * as Select by Pattern. Blank text keeps every file.
 */
export function filterFiles(files: File[], text: string): File[] {
	const trimmed = text.trim();
	if (!trimmed) return files;
	if (GLOB_CHARS.test(trimmed)) {
		const matcher = globMatcher(trimmed);
		return matcher ? files.filter((file) => matcher.test(fileDisplayName(file))) : files;
	}
	const needle = trimmed.toLocaleLowerCase();
	return files.filter((file) => fileDisplayName(file).toLocaleLowerCase().includes(needle));
}
