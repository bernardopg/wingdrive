import {toast} from '@wingdrive/primitives';
import {useCallback} from 'react';
import {usePlatform} from '../../../contexts/PlatformContext';
import {useLibraryMutation} from '../../../contexts/WingDriveContext';
import {useRefetchFileListings} from '../../../hooks/useRefetchFileListings';
import {useUndo} from '../../../hooks/useUndo';
import {useExplorer} from '../context';
import {useSelection} from '../SelectionContext';
import {i18n, useTranslation} from '../../../i18n';

/** First of "Untitled Folder", "Untitled Folder 2", ... not already taken. */
export function nextFolderName(existing: Iterable<string>): string {
	const baseName = i18n.t('explorer:create.folderBaseName');
	const taken = new Set(existing);
	if (!taken.has(baseName)) return baseName;
	let n = 2;
	while (taken.has(`${baseName} ${n}`)) n++;
	return `${baseName} ${n}`;
}

/**
 * Creates a new folder in the operational directory with a free default name.
 * Resolves to the folder name, or null when nothing was created.
 */
export function useCreateFolder() {
	const {t} = useTranslation('explorer');
	// currentFiles lists the operational directory whenever it is set, so the
	// free-name check runs against the folder that receives the new one.
	const {operationalPath, currentFiles} = useExplorer();
	const createFolder = useLibraryMutation('files.createFolder');
	const refetchListings = useRefetchFileListings();
	const platform = usePlatform();
	const undo = useUndo();
	// Every entry point (menu, shortcut) names the folder right away.
	const {renameWhenListed} = useSelection();

	return useCallback(async (): Promise<string | null> => {
		if (!operationalPath) return null;
		const name = nextFolderName(currentFiles.map((f) => f.name));
		try {
			const output = await createFolder.mutateAsync({
				parent: operationalPath,
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
					undo.record(t('undo.newFolder'), () => remove(path, expected));
				} catch {
					toast.error(
				t('errors.folderCreatedUndoUnavailable')
					);
				}
			}
			// The mutation creates the folder without emitting a listing
			// event; without the manual refetch the new folder only appeared
			// after leaving and re-entering.
			refetchListings();
			renameWhenListed(name);
			return name;
		} catch (err) {
			toast.error(t('errors.failedToCreateFolder', {error: err}));
			return null;
		}
	}, [
		operationalPath,
		currentFiles,
		createFolder,
		refetchListings,
		platform,
		undo,
		renameWhenListed,
		t
	]);
}
