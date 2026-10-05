import type {File} from '@wingdrive/ts-client';
import {useEffect, useState} from 'react';
import {useClipboard} from '../../../hooks/useClipboard';
import {useKeybind} from '../../../hooks/useKeybind';
import {useKeybindScope} from '../../../hooks/useKeybindScope';
import {useRefetchFileListings} from '../../../hooks/useRefetchFileListings';
import {useUndo} from '../../../hooks/useUndo';
import {isInputFocused} from '../../../util/keybinds/platform';
import {useExplorer} from '../context';
import {selectionCapabilities} from '../fileCapabilities';
import {useSelection} from '../SelectionContext';
import {useCreateFolder} from './useCreateFolder';
import {useDeleteFiles} from './useDeleteFiles';
import {useDuplicateFiles} from './useDuplicateFiles';
import {useOpenFile} from './useOpenFile';
import {usePasteFiles} from './usePasteFiles';
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
		isRenaming
	} = useSelection();
	const clipboard = useClipboard();
	const pasteFiles = usePasteFiles();
	const openFile = useOpenFile();
	const undo = useUndo();
	useKeybind('explorer.undo', undo.undo, {enabled: undo.canUndo});
	const {deleteFiles, isPending: isDeleting} = useDeleteFiles();
	const {duplicateFiles, isPending: isDuplicating} = useDuplicateFiles();
	const createFolder = useCreateFolder();
	const refetchListings = useRefetchFileListings();
	// Name of a folder created from the keyboard, renamed once it is listed
	const [pendingRename, setPendingRename] = useState<string | null>(null);

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

	useKeybind('explorer.newFolder', async () => {
		const name = await createFolder();
		if (name) setPendingRename(name);
	});

	useKeybind('explorer.toggleHiddenFiles', () => {
		setViewSettings({showHiddenFiles: !viewSettings.showHiddenFiles});
	});

	useEffect(() => {
		if (!pendingRename) return;
		const folder = files.find((f: File) => f.name === pendingRename);
		if (!folder) return;
		// startRename only accepts a single selection, so select first and
		// start the rename on the next run once the selection is committed.
		if (selectedFiles.length !== 1 || selectedFiles[0].id !== folder.id) {
			setSelectedFiles([folder]);
			return;
		}
		setPendingRename(null);
		startRename(folder.id);
	}, [files, pendingRename, selectedFiles, setSelectedFiles, startRename]);

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

	useKeybind(
		'explorer.openFile',
		() => {
			if (selectedFiles.length === 1) void openFile(selectedFiles[0]);
		},
		{enabled: selectedFiles.length === 1}
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
