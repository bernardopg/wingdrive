import type {ArchiveFormat} from '@wingdrive/ts-client';
import {i18n} from '../../../i18n';

/** Suffixes WingDrive extracts, longest first so `.tar.gz` wins over `.gz`. */
const ARCHIVE_SUFFIXES = [
	'.tar.gz',
	'.tar.bz2',
	'.tar.xz',
	'.tar.zst',
	'.tgz',
	'.tbz2',
	'.tbz',
	'.txz',
	'.tzst',
	'.tar',
	'.zip',
	'.jar',
	'.7z'
];

export function isArchiveName(name: string): boolean {
	const lower = name.toLowerCase();
	return ARCHIVE_SUFFIXES.some((suffix) => lower.endsWith(suffix) && lower.length > suffix.length);
}

/**
 * The archive file `path` lies inside, from the names of its folders, or the
 * path itself with `orSelf` when it names an archive.
 *
 * shortcut: a real folder named like an archive reads as one here; the daemon
 * checks the disk, so opening still works, only the menu is narrowed.
 */
export function containingArchive(path: string, orSelf = false): string | null {
	const parts = path.split('/');
	const last = orSelf ? parts.length : parts.length - 1;
	for (let i = 1; i < last; i++) {
		if (isArchiveName(parts[i])) return parts.slice(0, i + 1).join('/');
	}
	return null;
}

export const COMPRESS_FORMATS: {format: ArchiveFormat; label: string}[] = [
	{format: 'zip', label: 'archive.zip'},
	{format: 'tar_gz', label: 'archive.tarGz'},
	{format: 'tar_xz', label: 'archive.tarXz'},
	{format: 'tar_zst', label: 'archive.tarZst'},
	{format: 'tar_bz2', label: 'archive.tarBz2'}
];

/** Archive name for a selection: the item's own name, or "Archive" for several. */
export function archiveBaseName(names: string[]): string {
	if (names.length !== 1) return i18n.t('explorer:archive.baseName');
	const name = names[0];
	const dot = name.lastIndexOf('.');
	return dot > 0 ? name.slice(0, dot) : name;
}
