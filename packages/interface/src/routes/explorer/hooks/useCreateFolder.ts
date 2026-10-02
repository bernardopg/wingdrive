import {toast} from '@wingdrive/primitives';
import {useCallback} from 'react';
import {usePlatform} from '../../../contexts/PlatformContext';
import {useLibraryMutation} from '../../../contexts/WingDriveContext';
import {useRefetchFileListings} from '../../../hooks/useRefetchFileListings';
import {useUndo} from '../../../hooks/useUndo';
import {useExplorer} from '../context';

const BASE_NAME = 'Untitled Folder';

/** First of "Untitled Folder", "Untitled Folder 2", ... not already taken. */
export function nextFolderName(existing: Iterable<string>): string {
	const taken = new Set(existing);
	if (!taken.has(BASE_NAME)) return BASE_NAME;
	let n = 2;
	while (taken.has(`${BASE_NAME} ${n}`)) n++;
	return `${BASE_NAME} ${n}`;
}

/**
 * Creates a new folder in the current directory with a free default name.
 * Resolves to the folder name, or null when nothing was created.
 */
export function useCreateFolder() {
	const {currentPath, currentFiles} = useExplorer();
	const createFolder = useLibraryMutation('files.createFolder');
	const refetchListings = useRefetchFileListings();
	const platform = usePlatform();
	const undo = useUndo();

	return useCallback(async (): Promise<string | null> => {
		if (!currentPath) return null;
		const name = nextFolderName(currentFiles.map((f) => f.name));
		try {
			const output = await createFolder.mutateAsync({
				parent: currentPath,
				name,
				items: []
			});
			if (
				'Physical' in output.folder_path &&
				undo.isLocalPath(output.folder_path) &&
				platform.fileIdentity &&
				platform.undoEmptyFolder
			) {
				const path = output.folder_path.Physical.path;
				const remove = platform.undoEmptyFolder;
				try {
					const expected = await platform.fileIdentity(path);
					undo.record('new folder', () => remove(path, expected));
				} catch {
					toast.error(
						'Folder created successfully, but undo is unavailable'
					);
				}
			}
			// The mutation creates the folder without emitting a listing
			// event; without the manual refetch the new folder only appeared
			// after leaving and re-entering.
			refetchListings();
			return name;
		} catch (err) {
			toast.error(`Failed to create folder: ${err}`);
			return null;
		}
	}, [
		currentPath,
		currentFiles,
		createFolder,
		refetchListings,
		platform,
		undo
	]);
}
