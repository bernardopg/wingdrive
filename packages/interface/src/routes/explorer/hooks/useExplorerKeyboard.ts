import type {DirectorySortBy, File} from '@wingdrive/ts-client';
import {useEffect, useState} from 'react';
import {useNormalizedQuery} from '../../../contexts/WingDriveContext';
import {useClipboard} from '../../../hooks/useClipboard';
import {useKeybind} from '../../../hooks/useKeybind';
import {useKeybindScope} from '../../../hooks/useKeybindScope';
import {useRefetchFileListings} from '../../../hooks/useRefetchFileListings';
import {useUndo} from '../../../hooks/useUndo';
import {isInputFocused} from '../../../util/keybinds/platform';
import {useExplorer} from '../context';
import {useSelection} from '../SelectionContext';
import {useCreateFolder} from './useCreateFolder';
import {useDeleteFiles} from './useDeleteFiles';
import {useDuplicateFiles} from './useDuplicateFiles';
import {useOpenFile} from './useOpenFile';
import {usePasteFiles} from './usePasteFiles';
import {useTypeaheadSearch} from './useTypeaheadSearch';

export function useExplorerKeyboard() {
	const {
		currentPath,
		sortBy,
		sortDirection,
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

	// Query files for keyboard operations
	const directoryQuery = useNormalizedQuery({
		query: 'files.directory_listing',
		input: currentPath
			? {
					path: currentPath,
					limit: null,
					include_hidden: viewSettings.showHiddenFiles,
					sort_by: sortBy as DirectorySortBy,
					folders_first: viewSettings.foldersFirst,
					sort_direction: sortDirection
				}
			: null!,
		resourceType: 'file',
		enabled: !!currentPath,
		pathScope: currentPath ?? undefined,
		// First visit to a non-indexed folder returns an empty listing while
		// the ephemeral indexer warms up; poll briefly until rows appear.
		refetchInterval: (query) =>
			query.state.data && query.state.data.files.length === 0
				? 750
				: false
	});

	const files = (directoryQuery.data as any)?.files || [];

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

	// Copy: Store selected files in clipboard
	useKeybind(
		'explorer.copy',
		() => {
			if (selectedFiles.length === 0) return;
			const sdPaths = selectedFiles.map((f) => f.wing_path);
			clipboard.copyFiles(
				sdPaths,
				currentPath,
				selectedFiles.every((file) => file.is_local)
			);
		},
		{enabled: selectedFiles.length > 0}
	);

	// Cut: Store selected files in clipboard with cut operation
	useKeybind(
		'explorer.cut',
		() => {
			if (selectedFiles.length === 0) return;
			const sdPaths = selectedFiles.map((f) => f.wing_path);
			clipboard.cutFiles(
				sdPaths,
				currentPath,
				selectedFiles.every((file) => file.is_local)
			);
		},
		{enabled: selectedFiles.length > 0}
	);

	useKeybind('explorer.paste', () => pasteFiles(currentPath), {
		enabled: clipboard.canPaste() && !!currentPath
	});

	// Rename: Enter key triggers rename mode for any selected file or directory
	useKeybind(
		'explorer.renameFile',
		() => {
			if (selectedFiles.length === 1 && !isRenaming) {
				startRename(selectedFiles[0].id);
			}
		},
		{enabled: selectedFiles.length === 1 && !isRenaming}
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

	// Duplicate: Cmd+D duplicates selected files in place
	useKeybind(
		'explorer.duplicate',
		async () => {
			await duplicateFiles(selectedFiles);
		},
		{
			enabled:
				selectedFiles.length > 0 &&
				!isDuplicating &&
				selectedFiles.every((f) => 'Physical' in f.wing_path)
		}
	);

	// Delete: Move to trash
	useKeybind(
		'explorer.delete',
		async () => {
			const ok = await deleteFiles(selectedFiles, false);
			if (ok) clearSelection();
		},
		{enabled: selectedFiles.length > 0 && !isDeleting}
	);

	// Permanent Delete: Shift+Delete / Cmd+Alt+Backspace
	useKeybind(
		'explorer.permanentDelete',
		async () => {
			const ok = await deleteFiles(selectedFiles, true);
			if (ok) clearSelection();
		},
		{enabled: selectedFiles.length > 0 && !isDeleting}
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
