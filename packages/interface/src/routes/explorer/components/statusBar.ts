import type {Device, File, Volume} from '@wingdrive/ts-client';

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

/** Whole-segment prefix test; `\` separates segments too, for Windows paths like `C:\Users`. */
function isUnder(path: string, mountPoint: string): boolean {
	const root = mountPoint.replace(/[\\/]+$/, '');
	if (root === '') return path.startsWith('/');
	if (!path.startsWith(root)) return false;
	const next = path.charAt(root.length);
	return next === '' || next === '/' || next === '\\';
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

/**
 * The id of the device a path's slug names. Explorer URLs opened from outside
 * the app name this device `local`, which no device carries as its slug.
 */
export function deviceIdForSlug(devices: Iterable<Device>, slug: string): string | null {
	for (const device of devices) {
		if (device.slug === slug || (slug === 'local' && device.is_current)) return device.id;
	}
	return null;
}
