import {toast} from '@wingdrive/primitives';
import type {File} from '@wingdrive/ts-client';
import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useMemo,
	useRef,
	useState,
	type ReactNode
} from 'react';
import {useTabManager} from '../../components/TabManager';
import {usePlatform} from '../../contexts/PlatformContext';
import {useLibraryMutation, useWingDriveClient} from '../../contexts/WingDriveContext';
import {useClipboard} from '../../hooks/useClipboard';
import {useRefetchFileListings} from '../../hooks/useRefetchFileListings';
import {useUndo} from '../../hooks/useUndo';
import {useWaitForJob} from '../../hooks/useWaitForJob';
import {
	reconcileSelectedFiles,
	sameFiles,
	selectionCapabilities
} from './fileCapabilities';

interface SelectionContextValue {
	selectedFiles: File[];
	selectedFileIds: Set<string>;
	isSelected: (fileId: string) => boolean;
	setSelectedFiles: (files: File[]) => void;
	selectFile: (
		file: File,
		files: File[],
		multi?: boolean,
		range?: boolean
	) => void;
	clearSelection: () => void;
	selectAll: (files: File[]) => void;
	focusedIndex: number;
	setFocusedIndex: (index: number) => void;
	moveFocus: (
		direction: 'up' | 'down' | 'left' | 'right',
		files: File[]
	) => void;
	// Rename state
	renamingFileId: string | null;
	startRename: (fileId: string) => void;
	cancelRename: () => void;
	saveRename: (newName: string) => Promise<void>;
	isRenaming: boolean;
	/**
	 * Rebuilds the selection from the displayed collection: drops entries that
	 * left it and refreshes the rest. The explorer calls it whenever the
	 * collection changes.
	 */
	restoreSelectionFromFiles: (files: File[]) => void;
}

export const SelectionContext = createContext<SelectionContextValue | null>(
	null
);

interface SelectionProviderProps {
	children: ReactNode;
	isActiveTab?: boolean;
}

export function SelectionProvider({
	children,
	isActiveTab = true
}: SelectionProviderProps) {
	const platform = usePlatform();
	const clipboard = useClipboard();
	const client = useWingDriveClient();
	const tabManager = useTabManager();
	const {activeTabId, getSelectionIds, updateSelectionIds} = tabManager;
	const renameFile = useLibraryMutation('files.rename');
	const waitForJob = useWaitForJob();
	const undo = useUndo();
	const refetchListings = useRefetchFileListings();

	// Local state for File objects (not serializable, can't be stored in TabManager)
	const [selectedFiles, setSelectedFilesInternal] = useState<File[]>([]);
	const [focusedIndex, setFocusedIndex] = useState(-1);
	const [, setLastSelectedIndex] = useState(-1);
	const [renamingFileId, setRenamingFileId] = useState<string | null>(null);

	// Mirrors selectedFiles so the callback can compute chained updates
	// without reading stale state between batched re-renders
	const selectedFilesRef = useRef<File[]>([]);
	useEffect(() => {
		selectedFilesRef.current = selectedFiles;
	}, [selectedFiles]);

	useEffect(() => {
		const clearOnLibraryChange = () => {
			selectedFilesRef.current = [];
			setSelectedFilesInternal([]);
			setFocusedIndex(-1);
		};
		client.on("library-changed", clearOnLibraryChange);
		return () => client.off("library-changed", clearOnLibraryChange);
	}, [client]);

	// Track the stored IDs for the active tab (separate from File objects)
	const storedIds = getSelectionIds(activeTabId);

	// Clear selection when activeTabId changes (we'll restore it when files load)
	useEffect(() => {
		setSelectedFilesInternal([]);
		setFocusedIndex(-1);
		setLastSelectedIndex(-1);
	}, [activeTabId]);

	// Wrapper for setSelectedFiles that syncs to TabManager
	// Supports both direct values and updater functions
	const setSelectedFiles = useCallback(
		(filesOrUpdater: File[] | ((prev: File[]) => File[])) => {
			// Compute next from the ref, not the updater form, so the TabManager
			// sync stays in the event handler instead of running inside React's
			// render phase (updater functions run during render in React 19)
			const nextFiles =
				typeof filesOrUpdater === 'function'
					? filesOrUpdater(selectedFilesRef.current)
					: filesOrUpdater;

			selectedFilesRef.current = nextFiles;
			updateSelectionIds(
				activeTabId,
				nextFiles.map((f) => f.id)
			);

			setSelectedFilesInternal(nextFiles);
		},
		[activeTabId, updateSelectionIds]
	);

	// Sync selected file IDs to platform (for cross-window state sharing)
	// Only sync for the active tab to avoid conflicts
	useEffect(() => {
		if (!isActiveTab) return;

		const fileIds = selectedFiles.map((f) => f.id);

		if (platform.setSelectedFileIds) {
			platform.setSelectedFileIds(fileIds).catch((err) => {
				console.error(
					'Failed to sync selected files to platform:',
					err
				);
			});
		}
	}, [selectedFiles, platform, isActiveTab]);

	// Update native menu items based on selection and clipboard state
	// Only update for active tab
	useEffect(() => {
		if (!isActiveTab) return;

		const capabilities = selectionCapabilities(selectedFiles);

		platform.updateMenuItems?.([
			// Copy, cut and paste stay enabled so text inputs keep working; they
			// route to file operations or the native clipboard based on focus.
			{id: 'duplicate', enabled: capabilities.canDuplicate},
			{id: 'rename', enabled: capabilities.canRename},
			{id: 'delete', enabled: capabilities.canDelete}
		]);
	}, [selectedFiles, clipboard, platform, isActiveTab]);

	const clearSelection = useCallback(() => {
		setSelectedFiles([]);
		setFocusedIndex(-1);
		setLastSelectedIndex(-1);
	}, [setSelectedFiles]);

	const selectAll = useCallback(
		(files: File[]) => {
			setSelectedFiles([...files]);
			setLastSelectedIndex(files.length - 1);
		},
		[setSelectedFiles]
	);

	const selectFile = useCallback(
		(file: File, files: File[], multi = false, range = false) => {
			const fileIndex = files.findIndex((f) => f.id === file.id);

			if (range) {
				setLastSelectedIndex((prevLastIndex) => {
					if (prevLastIndex !== -1) {
						const start = Math.min(prevLastIndex, fileIndex);
						const end = Math.max(prevLastIndex, fileIndex);
						const rangeFiles = files.slice(start, end + 1);

						setSelectedFiles((prev) => {
							// If there's already a multi-file selection, add the range (Finder behavior)
							if (prev.length > 1) {
								// Create a map for O(1) lookup
								const existingIds = new Set(
									prev.map((f) => f.id)
								);
								const combined = [...prev];

								// Add new range files that aren't already selected
								for (const rangeFile of rangeFiles) {
									if (!existingIds.has(rangeFile.id)) {
										combined.push(rangeFile);
									}
								}

								return combined;
							} else {
								// Single file or empty selection, replace with range
								return rangeFiles;
							}
						});
					}
					return fileIndex; // Update anchor to clicked file for next range
				});
				setFocusedIndex(fileIndex);
			} else if (multi) {
				setSelectedFiles((prev) => {
					const isSelected = prev.some((f) => f.id === file.id);
					if (isSelected) {
						return prev.filter((f) => f.id !== file.id);
					} else {
						return [...prev, file];
					}
				});
				setFocusedIndex(fileIndex);
				setLastSelectedIndex(fileIndex);
			} else {
				setSelectedFiles([file]);
				setFocusedIndex(fileIndex);
				setLastSelectedIndex(fileIndex);
			}
		},
		[setSelectedFiles]
	);

	const moveFocus = useCallback(
		(direction: 'up' | 'down' | 'left' | 'right', files: File[]) => {
			if (files.length === 0) return;

			setFocusedIndex((currentFocusedIndex) => {
				let newIndex = currentFocusedIndex;

				if (direction === 'up')
					newIndex = Math.max(0, currentFocusedIndex - 1);
				if (direction === 'down')
					newIndex = Math.min(
						files.length - 1,
						currentFocusedIndex + 1
					);
				if (direction === 'left')
					newIndex = Math.max(0, currentFocusedIndex - 1);
				if (direction === 'right')
					newIndex = Math.min(
						files.length - 1,
						currentFocusedIndex + 1
					);

				if (newIndex !== currentFocusedIndex) {
					setSelectedFiles([files[newIndex]]);
					setLastSelectedIndex(newIndex);
				}

				return newIndex;
			});
		},
		[setSelectedFiles]
	);

	// Rename functions
	const startRename = useCallback(
		(fileId: string) => {
			if (
				selectionCapabilities(selectedFiles).canRename &&
				selectedFiles[0].id === fileId
			) {
				setRenamingFileId(fileId);
			}
		},
		[selectedFiles]
	);

	const cancelRename = useCallback(() => {
		setRenamingFileId(null);
	}, []);

	const saveRename = useCallback(
		async (newName: string) => {
			if (!renamingFileId) return;

			const file = selectedFiles.find((f) => f.id === renamingFileId);
			if (!file) {
				setRenamingFileId(null);
				return;
			}

			// Don't submit if name is empty or unchanged
			const currentFullName = file.extension
				? `${file.name}.${file.extension}`
				: file.name;
			if (!newName.trim() || newName === currentFullName) {
				setRenamingFileId(null);
				return;
			}

			try {
				const {result} = await waitForJob(() =>
					renameFile.mutateAsync({
						target: file.wing_path,
						new_name: newName
					})
				);
				if (
					result.status !== 'completed' ||
					(result.output.type === 'FileMove' &&
						result.output.data.failed_count > 0)
				) {
					refetchListings();
					throw new Error(
						result.status === 'failed'
							? result.error
							: 'Rename did not complete successfully'
					);
				}
				if (
					file.is_local &&
					'Physical' in file.wing_path &&
					platform.fileIdentity &&
					platform.undoMove
				) {
					const oldPath = file.wing_path.Physical.path;
					const newPath =
						oldPath.slice(
							0,
							Math.max(
								oldPath.lastIndexOf('/'),
								oldPath.lastIndexOf('\\')
							) + 1
						) + newName;
					const move = platform.undoMove;
					try {
						const expected = await platform.fileIdentity(newPath);
						undo.record('rename', () =>
							move(newPath, oldPath, expected)
						);
					} catch {
						toast.error(
							'Renamed successfully, but undo is unavailable'
						);
					}
				}
				setRenamingFileId(null);
				// Renames change paths on disk; without a refetch the old name stays
				// visible until the next navigation.
				refetchListings();
			} catch (error) {
				// Keep in edit mode on error so user can retry
				console.error('Rename failed:', error);
				toast.error(
					`Rename failed: ${error instanceof Error ? error.message : String(error)}`
				);
				throw error;
			}
		},
		[
			renamingFileId,
			selectedFiles,
			renameFile,
			refetchListings,
			waitForJob,
			platform,
			undo
		]
	);

	// Cancel rename when selection changes
	useEffect(() => {
		if (
			renamingFileId &&
			!selectedFiles.some((f) => f.id === renamingFileId)
		) {
			setRenamingFileId(null);
		}
	}, [selectedFiles, renamingFileId]);

	// Use stored IDs for selection checking (allows highlighting before File objects are restored)
	const selectedFileIds = useMemo(() => new Set(storedIds), [storedIds]);

	// Stable function for checking if a file is selected
	const isSelected = useCallback(
		(fileId: string) => selectedFileIds.has(fileId),
		[selectedFileIds]
	);

	const storedIdsRef = useRef(storedIds);
	storedIdsRef.current = storedIds;

	const restoreSelectionFromFiles = useCallback(
		(files: File[]) => {
			const ids = storedIdsRef.current;
			const next = reconcileSelectedFiles(ids, files, selectedFilesRef.current);

			if (!sameFiles(selectedFilesRef.current, next)) {
				selectedFilesRef.current = next;
				setSelectedFilesInternal(next);
			}

			// An empty collection is usually a listing still loading, so keep the
			// stored ids for tab restore and only prune against real rows.
			if (
				files.length > 0 &&
				(next.length !== ids.length || next.some((f, i) => f.id !== ids[i]))
			) {
				updateSelectionIds(
					activeTabId,
					next.map((f) => f.id)
				);
			}
		},
		[activeTabId, updateSelectionIds]
	);

	const isRenaming = renamingFileId !== null;

	const value = useMemo(
		() => ({
			selectedFiles,
			selectedFileIds,
			isSelected,
			setSelectedFiles,
			selectFile,
			clearSelection,
			selectAll,
			focusedIndex,
			setFocusedIndex,
			moveFocus,
			// Rename state
			renamingFileId,
			startRename,
			cancelRename,
			saveRename,
			isRenaming,
			// Restore selection
			restoreSelectionFromFiles
		}),
		[
			selectedFiles,
			selectedFileIds,
			isSelected,
			setSelectedFiles,
			selectFile,
			clearSelection,
			selectAll,
			focusedIndex,
			moveFocus,
			renamingFileId,
			startRename,
			cancelRename,
			saveRename,
			isRenaming,
			restoreSelectionFromFiles
		]
	);

	return (
		<SelectionContext.Provider value={value}>
			{children}
		</SelectionContext.Provider>
	);
}

export function useSelection() {
	const context = useContext(SelectionContext);
	if (!context)
		throw new Error('useSelection must be used within SelectionProvider');
	return context;
}
