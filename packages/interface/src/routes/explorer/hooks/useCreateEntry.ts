import {toast} from '@wingdrive/primitives';
import type {File, WingPath} from '@wingdrive/ts-client';
import {useCallback} from 'react';

import {usePlatform} from '../../../contexts/PlatformContext';
import {useLibraryMutation} from '../../../contexts/WingDriveContext';
import {useRefetchFileListings} from '../../../hooks/useRefetchFileListings';
import {useUndo} from '../../../hooks/useUndo';
import {useExplorer} from '../context';
import {physicalPath} from '../fileCapabilities';
import {requestReveal} from '../pendingReveal';

/** First of `base`, `base 2`, ... (before the extension) that is not taken. */
export function nextFreeName(base: string, existing: Iterable<string>): string {
	const taken = new Set(existing);
	if (!taken.has(base)) return base;
	// Only a short alphanumeric suffix counts as an extension, so "a.txt (link)" has none.
	const match = /\.[A-Za-z0-9]{1,10}$/.exec(base);
	const dot = match && match.index > 0 ? match.index : base.length;
	const [stem, ext] = [base.slice(0, dot), base.slice(dot)];
	let n = 2;
	while (taken.has(`${stem} ${n}${ext}`)) n++;
	return `${stem} ${n}${ext}`;
}

/** The on-disk name, with extension; `File.name` omits the extension. */
function diskName(file: File): string {
	const path = physicalPath(file);
	return path ? path.slice(path.lastIndexOf('/') + 1) : file.name;
}

/**
 * Creates empty files and symbolic links in the operational directory.
 * The new entry is selected and, for files, renamed in place once the
 * listing shows it.
 */
export function useCreateEntry() {
	const {operationalPath, currentFiles} = useExplorer();
	const createFile = useLibraryMutation('files.createFile');
	const createSymlink = useLibraryMutation('files.createSymlink');
	const refetchListings = useRefetchFileListings();
	const platform = usePlatform();
	const undo = useUndo();

	const recordUndo = useCallback(
		async (path: WingPath, label: string) => {
			if (
				!('Physical' in path) ||
				!undo.isLocalPath(path) ||
				!platform.fileIdentity ||
				!platform.undoNewFile
			)
				return;
			const local = path.Physical.path;
			const remove = platform.undoNewFile;
			try {
				const expected = await platform.fileIdentity(local);
				undo.record(label, () => remove(local, expected));
			} catch {
				// The entry exists; only its undo is unavailable.
			}
		},
		[platform, undo]
	);

	const newFile = useCallback(async () => {
		if (!operationalPath) return;
		const name = nextFreeName(
			'Untitled File',
			currentFiles.map(diskName)
		);
		try {
			const {path} = await createFile.mutateAsync({parent: operationalPath, name});
			if ('Physical' in path) requestReveal(path.Physical.path, {rename: true});
			await recordUndo(path, 'new file');
			refetchListings();
		} catch (error) {
			toast.error(`Failed to create file: ${error}`);
		}
	}, [operationalPath, currentFiles, createFile, recordUndo, refetchListings]);

	const newLink = useCallback(
		async (target: File) => {
			if (!operationalPath) return;
			const name = nextFreeName(
				`${diskName(target)} (link)`,
				currentFiles.map(diskName)
			);
			try {
				const {path} = await createSymlink.mutateAsync({
					target: target.wing_path,
					parent: operationalPath,
					name
				});
				if ('Physical' in path) requestReveal(path.Physical.path);
				await recordUndo(path, 'new link');
				refetchListings();
			} catch (error) {
				toast.error(`Failed to create link: ${error}`);
			}
		},
		[operationalPath, currentFiles, createSymlink, recordUndo, refetchListings]
	);

	return {newFile, newLink};
}
