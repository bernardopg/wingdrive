import type {File, WingPath} from '@wingdrive/ts-client';

/**
 * The folder the native picker returned, on this machine. The picker only
 * browses this machine, so sources must be local files or cloud files; Copy
 * To on a cloud file downloads it.
 */
export function pickedDestination(files: File[], picked: string | string[] | null): WingPath | null {
	const directory = Array.isArray(picked) ? picked[0] : picked;
	const first = files[0];
	if (!directory || !first || !canPickDestination(files)) return null;
	const deviceSlug = 'Physical' in first.wing_path ? first.wing_path.Physical.device_slug : 'local';
	return {Physical: {device_slug: deviceSlug, path: directory}};
}

export function canPickDestination(files: File[]): boolean {
	return (
		files.length > 0 &&
		files.every((f) => ('Physical' in f.wing_path && f.is_local) || 'Cloud' in f.wing_path)
	);
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
