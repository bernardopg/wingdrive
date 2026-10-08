import type {ArchiveFormat} from '@wingdrive/ts-client';

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

export const COMPRESS_FORMATS: {format: ArchiveFormat; label: string}[] = [
	{format: 'zip', label: 'ZIP (.zip)'},
	{format: 'tar_gz', label: 'Gzip tarball (.tar.gz)'},
	{format: 'tar_xz', label: 'XZ tarball (.tar.xz)'},
	{format: 'tar_zst', label: 'Zstandard tarball (.tar.zst)'},
	{format: 'tar_bz2', label: 'Bzip2 tarball (.tar.bz2)'}
];

/** Archive name for a selection: the item's own name, or "Archive" for several. */
export function archiveBaseName(names: string[]): string {
	if (names.length !== 1) return 'Archive';
	const name = names[0];
	const dot = name.lastIndexOf('.');
	return dot > 0 ? name.slice(0, dot) : name;
}
