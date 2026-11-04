import type {File, WingPath} from '@wingdrive/ts-client';

/**
 * # Where a file drop goes, and whether it moves
 *
 * Explorer URLs name this device `local`, and listings copy the slug of the
 * URL they were opened with into every file, while sidebar locations and
 * device views use the device's real slug. Both refer to the same disk, so
 * comparisons treat `local` and `localSlug` as one device. Comparing them as
 * strings made every drop between such views guess Copy.
 */

/** Whether two device slugs name the same device. */
export function sameDevice(a: string, b: string, localSlug: string | undefined): boolean {
	if (a === b) return true;
	const here = (slug: string) => slug === 'local' || (!!localSlug && slug === localSlug);
	return here(a) && here(b);
}

function parentOf(path: string): string {
	const trimmed = path.replace(/\/+$/, '');
	return trimmed.slice(0, trimmed.lastIndexOf('/')) || '/';
}

/** Whether every file already sits directly in `destination`, so the drop changes nothing. */
export function alreadyIn(files: File[], destination: WingPath, localSlug: string | undefined): boolean {
	if (!('Physical' in destination)) return false;
	const folder = destination.Physical.path.replace(/\/+$/, '') || '/';
	return files.every((file) => {
		const path = file.wing_path;
		return (
			'Physical' in path &&
			sameDevice(path.Physical.device_slug, destination.Physical.device_slug, localSlug) &&
			parentOf(path.Physical.path) === folder
		);
	});
}

/**
 * The operation a drop starts with before modifier keys: Move on the same
 * device, as Linux file managers do, and Copy across devices or clouds.
 */
export function guessDropOperation(
	source: WingPath,
	destination: WingPath,
	localSlug: string | undefined
): 'copy' | 'move' {
	if (!('Physical' in source) || !('Physical' in destination)) return 'copy';
	return sameDevice(source.Physical.device_slug, destination.Physical.device_slug, localSlug)
		? 'move'
		: 'copy';
}
