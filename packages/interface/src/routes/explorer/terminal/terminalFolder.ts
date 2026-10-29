import type {WingPath} from '@wingdrive/ts-client';

/**
 * The folder a docked terminal on this device can enter for `path`, or null
 * when there is none: virtual views, search results, and folders that live on
 * another device, whose paths mean nothing to the local shell.
 */
export function localFolder(path: WingPath | null, localSlug: string | undefined): string | null {
	if (!path || !('Physical' in path)) return null;
	const {device_slug, path: folder} = path.Physical;
	if (device_slug !== 'local' && device_slug !== localSlug) return null;
	return folder.startsWith('/') ? folder : null;
}

/** Height the panel keeps while you drag its edge. */
export function clampTerminalHeight(height: number, available: number): number {
	const max = Math.max(MIN_TERMINAL_HEIGHT, Math.floor(available * 0.8));
	return Math.round(Math.min(Math.max(height, MIN_TERMINAL_HEIGHT), max));
}

export const MIN_TERMINAL_HEIGHT = 96;
export const DEFAULT_TERMINAL_HEIGHT = 240;
