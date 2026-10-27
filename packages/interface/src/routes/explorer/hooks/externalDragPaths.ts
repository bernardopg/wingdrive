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

/**
 * Whether the pointer reached the window edge. The webview keeps receiving the
 * pointer while the button is held, so leaving it shows as coordinates at or
 * past the viewport bounds.
 */
export function pointerLeftViewport(
	x: number,
	y: number,
	width: number,
	height: number
): boolean {
	return x <= 0 || y <= 0 || x >= width - 1 || y >= height - 1;
}
