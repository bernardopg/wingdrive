import type {File} from '@wingdrive/ts-client';

import {physicalPath} from './fileCapabilities';

// Paths waiting to be selected once a listing containing them loads, for
// launches such as `wingdrive ~/file.txt` or FileManager1.ShowItems.
const pending = new Set<string>();

export function requestReveal(path: string) {
	pending.add(path);
}

/** Returns the files in `files` that were requested, and forgets them. */
export function takeRevealedFiles(files: File[]): File[] {
	if (pending.size === 0) return [];
	const matches = files.filter((file) => {
		const path = physicalPath(file);
		return path !== undefined && pending.has(path);
	});
	for (const file of matches) pending.delete(physicalPath(file)!);
	return matches;
}

export const REVEAL_EVENT = 'wingdrive:reveal';

export interface RevealDetail {
	fileId: string;
	index: number;
}

/**
 * Scrolls a revealed file into view. Views that virtualize rows also listen
 * for REVEAL_EVENT, because their element may not exist yet.
 */
export function scrollToRevealed(detail: RevealDetail) {
	requestAnimationFrame(() => {
		window.dispatchEvent(new CustomEvent<RevealDetail>(REVEAL_EVENT, {detail}));
		document
			.querySelector(`[data-file-id="${CSS.escape(detail.fileId)}"]`)
			?.scrollIntoView({block: 'center'});
	});
}
