import {
	ArrowCounterClockwise,
	Copy,
	FilePlus,
	FolderPlus,
	MagnifyingGlass,
	SelectionAll,
	SelectionInverse,
	TerminalWindow
} from '@phosphor-icons/react';
import {useDuplicatesDialog} from '../../../components/modals/DuplicatesModal';
import {useClipboard} from '../../../hooks/useClipboard';
import {useContextMenu} from '../../../hooks/useContextMenu';
import {useUndo} from '../../../hooks/useUndo';
import {useExplorer} from '../context';
import {useSelection} from '../SelectionContext';
import {useCreateEntry} from './useCreateEntry';
import {useCreateFolder} from './useCreateFolder';
import {useOpenTerminal} from './useOpenTerminal';
import {useSelectionCommands} from './useSelectionCommands';
import {usePasteFiles} from './usePasteFiles';
import {useTranslation} from '../../../i18n';

export function useEmptySpaceContextMenu() {
	const {t} = useTranslation('explorer');
	const {operationalPath, currentFiles} = useExplorer();
	const {selectAll} = useSelection();
	const selectionCommands = useSelectionCommands();
	const createFolder = useCreateFolder();
	const clipboard = useClipboard();
	const pasteFiles = usePasteFiles();
	const undo = useUndo();
	const openTerminal = useOpenTerminal();
	const {newFile} = useCreateEntry();
	const openDuplicates = useDuplicatesDialog();

	return useContextMenu({
		items: [
			{
				icon: ArrowCounterClockwise,
			label: t('contextMenu.undo'),
				onClick: undo.undo,
				keybindId: 'explorer.undo',
				condition: () => undo.canUndo
			},
			{
				icon: FolderPlus,
			label: t('contextMenu.newFolder'),
				onClick: createFolder,
				keybindId: 'explorer.newFolder',
				condition: () => !!operationalPath
			},
			{
				icon: FilePlus,
			label: t('contextMenu.newFile'),
				onClick: () => {
					void newFile();
				},
				keybindId: 'explorer.newFile',
				condition: () => !!operationalPath && 'Physical' in operationalPath
			},
			{
				icon: Copy,
			label: t('contextMenu.paste'),
				onClick: () => {
					void pasteFiles(operationalPath);
				},
				keybindId: 'explorer.paste',
				condition: () => clipboard.canPaste() && !!operationalPath
			},
			{
				icon: TerminalWindow,
			label: t('contextMenu.openTerminalHere'),
				onClick: () => {
					void openTerminal?.(operationalPath);
				},
				keybindId: 'explorer.openTerminal',
				condition: () =>
					!!openTerminal && !!operationalPath && 'Physical' in operationalPath
			},
			{
				icon: MagnifyingGlass,
			label: t('contextMenu.findDuplicates'),
				onClick: () => {
					if (operationalPath && 'Physical' in operationalPath) {
						openDuplicates(operationalPath.Physical.path);
					}
				},
				condition: () => !!operationalPath && 'Physical' in operationalPath
			},
			{type: 'separator'},
			{
				icon: SelectionAll,
				label: t('contextMenu.selectAll'),
				onClick: () => selectAll(currentFiles),
				keybindId: 'explorer.selectAll',
				condition: () => selectionCommands.hasFiles
			},
			{
				icon: SelectionInverse,
				label: t('contextMenu.invertSelection'),
				onClick: selectionCommands.invert,
				keybindId: 'explorer.invertSelection',
				condition: () => selectionCommands.hasFiles
			},
			{
				icon: SelectionAll,
				label: t('contextMenu.selectByPattern'),
				onClick: selectionCommands.selectByPattern,
				keybindId: 'explorer.selectByPattern',
				condition: () => selectionCommands.hasFiles
			}
		]
	});
}
