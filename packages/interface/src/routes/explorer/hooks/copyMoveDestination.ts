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

/**
 * The folder holding the first selected file, where the picker opens. Splits on
 * either separator because Windows paths come back with backslashes.
 */
export function pickerStartFolder(files: File[]): string | undefined {
	const first = files[0];
	if (!first || !('Physical' in first.wing_path)) return undefined;
	const path = first.wing_path.Physical.path.replace(/[\\/]+$/, '');
	const cut = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
	if (cut < 0) return undefined;
	const parent = path.slice(0, cut);
	// Keep the root's separator: `/` on Unix, `C:\` on Windows.
	if (parent === '') return path[cut];
	if (/^[A-Za-z]:$/.test(parent)) return `${parent}${path[cut]}`;
	return parent;
}
