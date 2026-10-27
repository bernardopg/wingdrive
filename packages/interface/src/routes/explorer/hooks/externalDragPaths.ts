import type {File} from '@wingdrive/ts-client';
import {isVirtualFile} from '@wingdrive/ts-client';

/** Resolve a drag only when every selected item exists on this device. */
export function externalDragPaths(
	file: File,
	selectedFiles: File[],
	localSlug?: string
): string[] {
	const files = selectedFiles.some((selected) => selected.id === file.id)
		? selectedFiles
		: [file];
	const paths: string[] = [];
	for (const item of files) {
		const path = item.wing_path;
		if (
			isVirtualFile(item) ||
			!('Physical' in path) ||
			(path.Physical.device_slug !== 'local' &&
				path.Physical.device_slug !== localSlug) ||
			!path.Physical.path
		) return [];
		paths.push(path.Physical.path);
	}
	return paths;
}
