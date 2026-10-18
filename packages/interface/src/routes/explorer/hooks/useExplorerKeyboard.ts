import type {File} from '@wingdrive/ts-client';
import {useEffect} from 'react';
import {useClipboard} from '../../../hooks/useClipboard';
import {useKeybind} from '../../../hooks/useKeybind';
import {useKeybindScope} from '../../../hooks/useKeybindScope';
import {useRefetchFileListings} from '../../../hooks/useRefetchFileListings';
import {useUndo} from '../../../hooks/useUndo';
import {isInputFocused} from '../../../util/keybinds/platform';
import {useExplorer} from '../context';
import {selectionCapabilities} from '../fileCapabilities';
import {useCreateEntry} from './useCreateEntry';
import {useOpenTerminal} from './useOpenTerminal';
import {useOpenInNewWindow} from './useOpenInNewWindow';
import {useSelection} from '../SelectionContext';
import {useCreateFolder} from './useCreateFolder';
import {useDeleteFiles} from './useDeleteFiles';
import {useDuplicateFiles} from './useDuplicateFiles';
import {useOpenFiles} from './useOpenFile';
import {usePasteFiles} from './usePasteFiles';
import {useSelectionCommands} from './useSelectionCommands';
import {useTypeaheadSearch} from './useTypeaheadSearch';

export function useExplorerKeyboard() {
	const {
		currentFiles,
		operationalPath,
		navigateToPath,
		goBack,
		goForward,
		canGoBack,
		canGoForward,
		viewMode,
		viewSettings,
		setViewSettings,
		sidebarVisible,
		inspectorVisible,
		openQuickPreview,
		tagModeActive,
		setTagModeActive
	} = useExplorer();
	const {
		selectedFiles,
		selectAll,
		clearSelection,
		focusedIndex,
		setFocusedIndex,
		setSelectedFiles,
		startRename,
		isRenaming,
		pendingRename,
		renameWhenListed
	} = useSelection();
	const clipboard = useClipboard();
	const pasteFiles = usePasteFiles();
	const openFiles = useOpenFiles();
	const undo = useUndo();
	useKeybind('explorer.undo', undo.undo, {enabled: undo.canUndo});
	const {deleteFiles, isPending: isDeleting} = useDeleteFiles();
	const {duplicateFiles, isPending: isDuplicating} = useDuplicateFiles();
	const createFolder = useCreateFolder();
	const refetchListings = useRefetchFileListings();

	// Activate explorer keybind scope when this hook is active
	useKeybindScope('explorer');
	useKeybind('explorer.navigateBack', goBack, {enabled: canGoBack});
	useKeybind('explorer.navigateForward', goForward, {enabled: canGoForward});

	// The displayed collection, so select-all and typeahead never reach rows
	// another view or an older listing would show.
	const files = currentFiles;
	const capabilities = selectionCapabilities(selectedFiles);
	const targets = capabilities.operable;

	// Typeahead search (disabled for column view - it handles its own)
	const typeahead = useTypeaheadSearch({
		files,
		onMatch: (file, index) => {
			setFocusedIndex(index);
			setSelectedFiles([file]);
		},
		enabled: viewMode !== 'column'
	});

	useKeybind('explorer.refresh', refetchListings);

	useKeybind('explorer.newFolder', () => {
		void createFolder();
	});

	const {newFile} = useCreateEntry();
	useKeybind('explorer.newFile', () => {
		void newFile();
	});

	const openTerminal = useOpenTerminal();
	useKeybind('explorer.openTerminal', () => {
		void openTerminal?.(operationalPath);
	});

	const openInNewWindow = useOpenInNewWindow();
	useKeybind('explorer.openInNewWindow', () => {
		void openInNewWindow?.(operationalPath);
	});

	const selectionCommands = useSelectionCommands();
	useKeybind('explorer.invertSelection', selectionCommands.invert, {enabled: selectionCommands.hasFiles});
	useKeybind('explorer.selectByPattern', selectionCommands.selectByPattern, {
		enabled: selectionCommands.hasFiles
	});

	useKeybind('explorer.toggleHiddenFiles', () => {
		setViewSettings({showHiddenFiles: !viewSettings.showHiddenFiles});
	});

	useEffect(() => {
		if (!pendingRename) return;
		const folder = files.find((f: File) => f.name === pendingRename);
		if (!folder) return;
		renameWhenListed(null);
		// Passing the file selects it and starts the rename in one step; the
		// selection ref startRename reads otherwise still holds the old one here.
		startRename(folder.id, folder);
	}, [files, pendingRename, renameWhenListed, startRename]);

	useKeybind(
		'explorer.copy',
		() => {
			clipboard.copyFiles(
				targets.map((f) => f.wing_path),
				operationalPath,
				targets.every((file) => file.is_local)
			);
		},
		{enabled: capabilities.canCopy}
	);

	useKeybind(
		'explorer.cut',
		() => {
			clipboard.cutFiles(
				targets.map((f) => f.wing_path),
				operationalPath,
				targets.every((file) => file.is_local)
			);
		},
		{enabled: capabilities.canCopy}
	);

	useKeybind('explorer.paste', () => pasteFiles(operationalPath), {
		enabled: clipboard.canPaste() && !!operationalPath
	});

	useKeybind(
		'explorer.renameFile',
		() => {
			if (capabilities.canRename && !isRenaming) {
				startRename(targets[0].id);
			}
		},
		{enabled: capabilities.canRename && !isRenaming}
	);

	// Tag mode: T key enters tag assignment mode
	useKeybind(
		'explorer.enterTagMode',
		() => {
			setTagModeActive(true);
		},
		{enabled: !tagModeActive}
	);

	// Quick Preview: Spacebar opens quick preview
	useKeybind(
		'explorer.toggleQuickPreview',
		() => {
			if (selectedFiles.length === 1) {
				openQuickPreview(selectedFiles[0].id);
			}
		},
		{enabled: selectedFiles.length === 1}
	);

	const openSelection = () => void openFiles(selectedFiles);
	useKeybind('explorer.openFile', openSelection, {
		enabled: selectedFiles.length > 0
	});
	useKeybind(
		'explorer.openSelection',
		() => {
			// Enter on a focused button or menu item belongs to that control.
			if (focusIsControl()) return;
			openSelection();
		},
		{enabled: selectedFiles.length > 0 && !isRenaming}
	);

	useKeybind(
		'explorer.duplicate',
		async () => {
			await duplicateFiles(targets);
		},
		{enabled: capabilities.canDuplicate && !isDuplicating}
	);

	useKeybind(
		'explorer.delete',
		async () => {
			const ok = await deleteFiles(targets, false);
			if (ok) clearSelection();
		},
		{enabled: capabilities.canDelete && !isDeleting}
	);

	useKeybind(
		'explorer.permanentDelete',
		async () => {
			const ok = await deleteFiles(targets, true);
			if (ok) clearSelection();
		},
		{enabled: capabilities.canDelete && !isDeleting}
	);

	useEffect(() => {
		const handleKeyDown = async (e: KeyboardEvent) => {
			// Skip all keyboard shortcuts if renaming or typing in an input
			if (isRenaming || isInputFocused()) return;

			// Arrow keys: Navigation
			if (
				['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(
					e.key
				)
			) {
				// Skip views that handle their own keyboard navigation
				if (
					viewMode === 'column' ||
					viewMode === 'media' ||
					viewMode === 'grid' ||
					viewMode === 'list'
				) {
					return;
				}

				e.preventDefault();

				if (files.length === 0) return;
			}

			// Cmd/Ctrl+A: Select all
			if ((e.metaKey || e.ctrlKey) && e.key === 'a') {
				e.preventDefault();
				selectAll(files);
				return;
			}

			// Escape: Clear selection
			if (e.code === 'Escape' && selectedFiles.length > 0) {
				clearSelection();
			}

			// Typeahead search (handled by hook, disabled for column view)
			typeahead.handleKey(e);
		};

		window.addEventListener('keydown', handleKeyDown);
		return () => {
			window.removeEventListener('keydown', handleKeyDown);
			typeahead.cleanup();
		};
	}, [
		selectedFiles,
		files,
		focusedIndex,
		viewMode,
		viewSettings,
		sidebarVisible,
		inspectorVisible,
		selectAll,
		clearSelection,
		navigateToPath,
		setFocusedIndex,
		setSelectedFiles,
		openQuickPreview,
		isRenaming,
		typeahead
	]);
}

/**
 * Whether keyboard focus is on a control that handles Enter itself.
 * `[role="button"]` is left out: dnd-kit gives every file item that role.
 */
function focusIsControl(): boolean {
	const active = document.activeElement;
	return (
		active instanceof HTMLElement &&
		active.closest(
			'button, a[href], [role="menuitem"], [role="columnheader"], [role="tab"]'
		) !== null
	);
}
