import type {File} from '@wingdrive/ts-client';

import {physicalPath} from './fileCapabilities';

// Paths waiting to be selected once a listing containing them loads, for
// launches such as `wingdrive ~/file.txt` or FileManager1.ShowItems.
const pending = new Set<string>();
const renameOnReveal = new Set<string>();

/** Selects `path` once it appears; `rename` also starts renaming it. */
export function requestReveal(path: string, options?: {rename?: boolean}) {
	pending.add(path);
	if (options?.rename) renameOnReveal.add(path);
}

/** Whether the revealed path asked to be renamed, consuming the request. */
export function takeRenameRequest(path: string): boolean {
	return renameOnReveal.delete(path);
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
 * Scrolls a revealed file into view. Large listings render their items a few
 * frames after the data arrives, so the lookup retries for up to a second.
 * Views that virtualize rows also listen for REVEAL_EVENT, because their
 * element may never exist until they scroll.
 */
export function scrollToRevealed(detail: RevealDetail) {
	window.dispatchEvent(new CustomEvent<RevealDetail>(REVEAL_EVENT, {detail}));
	const selector = `[data-file-id="${CSS.escape(detail.fileId)}"]`;
	let attempts = 0;
	const tryScroll = () => {
		const element = document.querySelector(selector);
		if (element) element.scrollIntoView({block: 'center'});
		else if (++attempts < 20) setTimeout(tryScroll, 50);
	};
	requestAnimationFrame(tryScroll);
}
