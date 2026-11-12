import type {WingPath} from '@wingdrive/ts-client';

/**
 * Keys that identify the folder a column shows and the folder a file sits in,
 * for device folders and cloud folders alike. Cloud keys carry the volume so
 * two accounts with the same folder names never match.
 */
export function folderKey(path: WingPath | null | undefined): string {
	if (!path) return '';
	if ('Physical' in path) return path.Physical?.path || '';
	if ('Cloud' in path) {
		const {service, identifier, path: inner} = path.Cloud;
		return `${service}://${identifier}/${inner.replace(/^\/+|\/+$/g, '')}`;
	}
	return JSON.stringify(path);
}

/** Key of the folder that holds `path`, in the form `folderKey` returns; null when unknown. */
export function parentFolderKey(path: WingPath): string | null {
	if ('Physical' in path) {
		const full = path.Physical?.path;
		if (!full) return null;
		return full.substring(0, full.lastIndexOf('/'));
	}
	if ('Cloud' in path) {
		const {service, identifier, path: inner} = path.Cloud;
		const trimmed = inner.replace(/^\/+|\/+$/g, '');
		const cut = trimmed.lastIndexOf('/');
		return `${service}://${identifier}/${cut < 0 ? '' : trimmed.slice(0, cut)}`;
	}
	return null;
}

/** A React key for a column: its scope and folder. */
export function columnKey(path: WingPath, index: number): string {
	if ('Physical' in path) return `${path.Physical?.device_slug}-${path.Physical?.path}-${index}`;
	return `${folderKey(path)}-${index}`;
}
