import {
	ArrowCounterClockwise,
	Copy,
	FilePlus,
	FolderPlus,
	TerminalWindow
} from '@phosphor-icons/react';
import {useClipboard} from '../../../hooks/useClipboard';
import {useContextMenu} from '../../../hooks/useContextMenu';
import {useUndo} from '../../../hooks/useUndo';
import {useExplorer} from '../context';
import {useCreateEntry} from './useCreateEntry';
import {useCreateFolder} from './useCreateFolder';
import {useOpenTerminal} from './useOpenTerminal';
import {usePasteFiles} from './usePasteFiles';

export function useEmptySpaceContextMenu() {
	const {operationalPath} = useExplorer();
	const createFolder = useCreateFolder();
	const clipboard = useClipboard();
	const pasteFiles = usePasteFiles();
	const undo = useUndo();
	const openTerminal = useOpenTerminal();
	const {newFile} = useCreateEntry();

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
				icon: FilePlus,
				label: 'New File',
				onClick: () => {
					void newFile();
				},
				keybindId: 'explorer.newFile',
				condition: () => !!operationalPath && 'Physical' in operationalPath
			},
			{
				icon: Copy,
				label: 'Paste',
				onClick: () => {
					void pasteFiles(operationalPath);
				},
				keybindId: 'explorer.paste',
				condition: () => clipboard.canPaste() && !!operationalPath
			},
			{
				icon: TerminalWindow,
				label: 'Open Terminal Here',
				onClick: () => {
					void openTerminal?.(operationalPath);
				},
				keybindId: 'explorer.openTerminal',
				condition: () =>
					!!openTerminal && !!operationalPath && 'Physical' in operationalPath
			}
		]
	});
}
