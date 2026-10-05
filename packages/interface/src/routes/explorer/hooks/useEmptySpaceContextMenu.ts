import {ArrowCounterClockwise, Copy, FolderPlus} from '@phosphor-icons/react';
import {useClipboard} from '../../../hooks/useClipboard';
import {useContextMenu} from '../../../hooks/useContextMenu';
import {useUndo} from '../../../hooks/useUndo';
import {useExplorer} from '../context';
import {useCreateFolder} from './useCreateFolder';
import {usePasteFiles} from './usePasteFiles';

export function useEmptySpaceContextMenu() {
	const {operationalPath} = useExplorer();
	const createFolder = useCreateFolder();
	const clipboard = useClipboard();
	const pasteFiles = usePasteFiles();
	const undo = useUndo();

	return useContextMenu({
		items: [
			{
				icon: ArrowCounterClockwise,
				label: 'Undo',
				onClick: undo.undo,
				keybindId: 'explorer.undo',
				condition: () => undo.canUndo
			},
			{
				icon: FolderPlus,
				label: 'New Folder',
				onClick: createFolder,
				keybindId: 'explorer.newFolder',
				condition: () => !!operationalPath
			},
			{
				icon: Copy,
				label: 'Paste',
				onClick: () => {
					void pasteFiles(operationalPath);
				},
				keybindId: 'explorer.paste',
				condition: () => clipboard.canPaste() && !!operationalPath
			}
		]
	});
}
