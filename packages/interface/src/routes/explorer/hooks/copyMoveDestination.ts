import type {File, WingPath} from '@wingdrive/ts-client';

/**
 * The folder the native picker returned, on the sources' device. The picker
 * only browses this machine, so every source must be a local physical path.
 */
export function pickedDestination(files: File[], picked: string | string[] | null): WingPath | null {
	const directory = Array.isArray(picked) ? picked[0] : picked;
	const first = files[0];
	if (!directory || !first || !canPickDestination(files)) return null;
	if (!('Physical' in first.wing_path)) return null;
	return {Physical: {device_slug: first.wing_path.Physical.device_slug, path: directory}};
}

export function canPickDestination(files: File[]): boolean {
	return files.length > 0 && files.every((f) => 'Physical' in f.wing_path && f.is_local);
}
