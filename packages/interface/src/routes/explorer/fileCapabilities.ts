import type {File} from '@wingdrive/ts-client';
import {isVirtualFile} from '@wingdrive/ts-client';

/**
 * What the current selection allows. Virtual entries (devices, volumes and
 * locations shown as folders) carry the wing_path of the root they represent,
 * so passing them to copy, move, rename or delete would act on that whole
 * root. Any virtual entry in the selection disables every file command.
 */
export interface SelectionCapabilities {
	/** Real filesystem entries in the selection. */
	operable: File[];
	hasVirtual: boolean;
	canCopy: boolean;
	canDelete: boolean;
	canRename: boolean;
	canDuplicate: boolean;
	canOpen: boolean;
}

export function isOperableFile(file: File | null | undefined): file is File {
	return file != null && file.wing_path != null && !isVirtualFile(file);
}

export function selectionCapabilities(files: File[]): SelectionCapabilities {
	const hasVirtual = files.some((file) => isVirtualFile(file));
	const operable = files.filter(isOperableFile);
	const canModify = !hasVirtual && operable.length > 0;

	return {
		operable,
		hasVirtual,
		canCopy: canModify,
		canDelete: canModify,
		canRename: canModify && operable.length === 1,
		// Duplicates are written next to the source, which needs a local path.
		canDuplicate:
			canModify && operable.every((file) => 'Physical' in file.wing_path),
		canOpen: files.length === 1
	};
}

/**
 * Rebuilds the selection from the collection the user is looking at.
 *
 * Keeps the stored order, drops ids that are no longer displayed and swaps in
 * the latest File objects so commands never act on a path that was renamed,
 * moved or deleted since the selection was made.
 */
export function reconcileSelectedFiles(
	selectedIds: readonly string[],
	collection: readonly File[],
	previous: readonly File[] = []
): File[] {
	if (selectedIds.length === 0 || collection.length === 0) return [];
	const byId = new Map(collection.map((file) => [file.id, file]));
	const result: File[] = [];
	for (const id of selectedIds) {
		const file = byId.get(id) ?? findByPreviousPath(id, collection, previous);
		if (file) result.push(file);
	}
	return result;
}

function physicalPath(file: File): string | undefined {
	return 'Physical' in file.wing_path ? file.wing_path.Physical.path : undefined;
}

// UUID reconciliation can swap an ephemeral id for the persistent one while a
// file stays selected; the path it had last time still finds it.
function findByPreviousPath(
	id: string,
	collection: readonly File[],
	previous: readonly File[]
): File | undefined {
	const known = previous.find((file) => file.id === id);
	const path = known && physicalPath(known);
	return path ? collection.find((file) => physicalPath(file) === path) : undefined;
}

/** True when both lists hold the same File objects in the same order. */
export function sameFiles(a: readonly File[], b: readonly File[]): boolean {
	return a.length === b.length && a.every((file, index) => file === b[index]);
}
