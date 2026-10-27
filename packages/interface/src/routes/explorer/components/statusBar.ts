import type {File, Volume} from '@wingdrive/ts-client';

export interface FileSummary {
	folders: number;
	files: number;
	/** Summed size of the files; folders count only when their content size is known. */
	bytes: number;
}

/** Counts folders and files and adds up their sizes. */
export function summarizeFiles(files: File[]): FileSummary {
	const summary: FileSummary = {folders: 0, files: 0, bytes: 0};
	for (const file of files) {
		if (file.kind === 'Directory') {
			summary.folders++;
			summary.bytes += file.directory_size ?? 0;
		} else {
			summary.files++;
			summary.bytes += file.size;
		}
	}
	return summary;
}

function isUnder(path: string, mountPoint: string): boolean {
	if (mountPoint === '/') return path.startsWith('/');
	const root = mountPoint.replace(/\/+$/, '');
	return path === root || path.startsWith(`${root}/`);
}

/**
 * The mounted volume of the given device that holds the path: the one with the
 * longest mount point the path sits under, so `/home` wins over `/`.
 */
export function volumeForPath(
	volumes: Volume[],
	deviceId: string,
	path: string
): Volume | null {
	let best: {volume: Volume; length: number} | null = null;
	for (const volume of volumes) {
		if (volume.device_id !== deviceId || !volume.is_mounted) continue;
		for (const mountPoint of [volume.mount_point, ...volume.mount_points]) {
			if (!mountPoint || !isUnder(path, mountPoint)) continue;
			if (!best || mountPoint.length > best.length) {
				best = {volume, length: mountPoint.length};
			}
		}
	}
	return best?.volume ?? null;
}
