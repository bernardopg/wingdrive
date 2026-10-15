import {
	ArrowCounterClockwise,
	Copy,
	FilePlus,
	FolderPlus,
	MagnifyingGlass,
	TerminalWindow
} from '@phosphor-icons/react';
import {useDuplicatesDialog} from '../../../components/modals/DuplicatesModal';
import {useClipboard} from '../../../hooks/useClipboard';
import {useContextMenu} from '../../../hooks/useContextMenu';
import {useUndo} from '../../../hooks/useUndo';
import {useExplorer} from '../context';
import {useCreateEntry} from './useCreateEntry';
import {useCreateFolder} from './useCreateFolder';
import {useOpenTerminal} from './useOpenTerminal';
import {usePasteFiles} from './usePasteFiles';
import {useTranslation} from '../../../i18n';

export function useEmptySpaceContextMenu() {
	const {t} = useTranslation('explorer');
	const {operationalPath} = useExplorer();
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
			}
		]
	});
}
