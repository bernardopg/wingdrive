import {
	ArrowCounterClockwise,
	Copy,
	FilePlus,
	FileText,
	FolderPlus,
	MagnifyingGlass,
	SelectionAll,
	SelectionInverse,
	TerminalWindow,
	UploadSimple
} from '@phosphor-icons/react';
import {useDuplicatesDialog} from '../../../components/modals/DuplicatesModal';
import {useFileOperationDialog} from '../../../components/modals/FileOperationModal';
import {usePlatform} from '../../../contexts/PlatformContext';
import {useClipboard} from '../../../hooks/useClipboard';
import {useContextMenu} from '../../../hooks/useContextMenu';
import {useLibraryQuery} from '../../../contexts/WingDriveContext';
import {useUndo} from '../../../hooks/useUndo';
import {useExplorer} from '../context';
import {useSelection} from '../SelectionContext';
import {useCreateEntry} from './useCreateEntry';
import {useCreateFolder} from './useCreateFolder';
import {useOpenTerminal} from './useOpenTerminal';
import {useTerminalPanel} from './useTerminalPanel';
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
	const terminalPanel = useTerminalPanel();
	const {newFile, newFromTemplate} = useCreateEntry();
	// Small and cheap, and the user may add templates while the app runs.
	const {data: templatesData} = useLibraryQuery(
		{type: 'files.templates', input: {}},
		{staleTime: 30_000}
	);
	const templates = templatesData?.templates ?? [];
	const openDuplicates = useDuplicatesDialog();
	const platform = usePlatform();
	const openFileOperation = useFileOperationDialog();
	// Picking files and copying them into the cloud folder uploads them.
	const uploadHere = async () => {
		if (!operationalPath || !platform.openFilePickerDialog) return;
		const picked = await platform.openFilePickerDialog({title: t('contextMenu.uploadFiles'), multiple: true});
		const paths = Array.isArray(picked) ? picked : picked ? [picked] : [];
		if (paths.length === 0) return;
		openFileOperation({
			operation: 'copy',
			sources: paths.map((path) => ({Physical: {device_slug: 'local', path}})),
			destination: operationalPath
		});
	};

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
				icon: UploadSimple,
				label: t('contextMenu.uploadFiles'),
				onClick: () => void uploadHere(),
				condition: () =>
					!!operationalPath && 'Cloud' in operationalPath && !!platform.openFilePickerDialog
			},
			{
				icon: FilePlus,
			label: t('contextMenu.newFile'),
				onClick: () => {
					void newFile();
				},
				keybindId: 'explorer.newFile',
				condition: () =>
					!!operationalPath && ('Physical' in operationalPath || 'Cloud' in operationalPath)
			},
			{
				type: 'submenu',
				icon: FileText,
				label: t('contextMenu.newFromTemplate'),
				condition: () =>
					!!operationalPath &&
					('Physical' in operationalPath || 'Cloud' in operationalPath) &&
					templates.length > 0,
				submenu: templates.map((template) => ({
					label: template.name,
					onClick: () => {
						void newFromTemplate(template);
					}
				}))
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
				icon: TerminalWindow,
				label: terminalPanel.visible
					? t('contextMenu.hideTerminalPanel')
					: t('contextMenu.showTerminalPanel'),
				onClick: terminalPanel.toggle,
				keybindId: 'explorer.toggleTerminal',
				condition: () => terminalPanel.supported
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
