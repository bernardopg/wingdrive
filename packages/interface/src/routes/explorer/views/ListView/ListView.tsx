import {CaretDown, CaretUp} from '@phosphor-icons/react';
import {flexRender} from '@tanstack/react-table';
import {useVirtualizer} from '@tanstack/react-virtual';
import type {DirectorySortBy, File} from '@wingdrive/ts-client';
import clsx from 'clsx';
import {
	memo,
	useCallback,
	useEffect,
	useLayoutEffect,
	useMemo,
	useRef,
	useState
} from 'react';
import {useTabManager} from '../../../../components/TabManager';
import {useLibraryQueries} from '../../../../contexts/WingDriveContext';
import {isInputFocused} from '../../../../util/keybinds/platform';
import {useExplorer} from '../../context';
import {useEmptySpaceContextMenu} from '../../hooks/useEmptySpaceContextMenu';
import {useExplorerFiles} from '../../hooks/useExplorerFiles';
import {useFolderFilter} from '../../hooks/useFolderFilter';
import {REVEAL_EVENT, type RevealDetail} from '../../pendingReveal';
import {useTabScroll} from '../../hooks/useTabScroll';
import {useSelection} from '../../SelectionContext';
import {DragSelect} from './DragSelect';
import {vcsStatusOf} from '../../components/vcsStatus';
import {
	attachChildren,
	blocksByTopLevel,
	expandedWithin,
	physicalPathOf,
	toggleExpandedPath
} from './listExpansion';
import {TableRow} from './TableRow';
import {useExpandedFolderEvents} from './useExpandedFolderEvents';
import {GROUP_HEADER_HEIGHT, GroupHeader, useGroupLabel} from '../GroupHeader';
import {buildListLayout, groupFiles, isGroupBy} from '../grouping';
import {useTranslation} from '../../../../i18n';
import {
	ROW_HEIGHT,
	TABLE_HEADER_HEIGHT,
	TABLE_PADDING_X,
	TABLE_PADDING_Y,
	useTable
} from './useTable';

const NO_FILES: File[] = [];

export const ListView = memo(function ListView() {
	const {t} = useTranslation('explorer');
	const {setCurrentFiles, scrollPosition, viewSettings, currentPath, sortBy, sortDirection} =
		useExplorer();
	const {activeTabId, getExplorerState, updateExplorerState} = useTabManager();
	const groupLabel = useGroupLabel();
	const {
		focusedIndex,
		setFocusedIndex,
		selectedFileIds,
		isSelected,
		selectFile,
		moveFocus,
		selectedFiles,
		renamingFileId,
		saveRename,
		cancelRename
	} = useSelection();
	const rename = useMemo(
		() => ({onSave: saveRename, onCancel: cancelRename}),
		[saveRename, cancelRename]
	);

	const containerRef = useRef<HTMLDivElement>(null);
	const headerScrollRef = useRef<HTMLDivElement>(null);
	const bodyScrollRef = useRef<HTMLDivElement>(null);
	const emptySpaceContextMenu = useEmptySpaceContextMenu();
	const [availableWidth, setAvailableWidth] = useState(0);

	// Columns fill the view, so track its width (without the scrollbar).
	useLayoutEffect(() => {
		const container = containerRef.current;
		if (!container) return;
		setAvailableWidth(container.clientWidth);
		const observer = new ResizeObserver(() =>
			setAvailableWidth(container.clientWidth)
		);
		observer.observe(container);
		return () => observer.disconnect();
	}, []);

	// Get files from centralized hook (handles search, virtual, and directory)
	const {files: listedFiles, vcsStatus, source} = useExplorerFiles();
	const {files, unfilteredCount} = useFolderFilter(listedFiles);

	// Folders expanded in place, listed with the same options as the folder shown.
	const shownFolder =
		source === 'directory' && currentPath && 'Physical' in currentPath
			? currentPath.Physical.path
			: null;
	const storedExpanded = getExplorerState(activeTabId).listExpanded;
	const expandedPaths = useMemo(
		() => expandedWithin(storedExpanded ?? [], shownFolder),
		[storedExpanded, shownFolder]
	);
	const deviceSlug =
		currentPath && 'Physical' in currentPath ? currentPath.Physical.device_slug : 'local';
	const childListings = useLibraryQueries(
		'files.directory_listing',
		expandedPaths.map((path) => ({
			path: {Physical: {device_slug: deviceSlug, path}},
			limit: null,
			include_hidden: viewSettings.showHiddenFiles,
			sort_by: sortBy as DirectorySortBy,
			folders_first: viewSettings.foldersFirst,
			sort_direction: sortDirection
		})),
		{staleTime: 5_000}
	);
	const listingData = childListings.map((query) => query.data?.files);
	const listingKey = childListings.map((query) => query.dataUpdatedAt).join(',');
	const expansion = useMemo(
		() => attachChildren(files, expandedPaths, listingData),
		// listingData is a fresh array each render; its contents change with listingKey.
		// eslint-disable-next-line react-hooks/exhaustive-deps
		[files, expandedPaths, listingKey]
	);
	useExpandedFolderEvents(currentPath, !!shownFolder && expandedPaths.length > 0);
	// Each expanded folder's Git state, read like the shown folder's.
	const childVcsQueries = useLibraryQueries(
		'files.vcs_status',
		expandedPaths.map((path) => ({path: {Physical: {device_slug: deviceSlug, path}}})),
		{retry: false, staleTime: 5_000}
	);
	const childVcsKey = childVcsQueries.map((query) => query.dataUpdatedAt).join(',');
	const vcsStatusByFolder = useMemo(
		() =>
			new Map(expandedPaths.map((path, index) => [path, childVcsQueries[index]?.data])),
		// childVcsQueries is a fresh array each render; its contents change with childVcsKey.
		// eslint-disable-next-line react-hooks/exhaustive-deps
		[expandedPaths, childVcsKey]
	);
	const toggleExpanded = useCallback(
		(file: File) => {
			const path = physicalPathOf(file);
			if (!path) return;
			updateExplorerState(activeTabId, {
				listExpanded: toggleExpandedPath(
					expandedWithin(getExplorerState(activeTabId).listExpanded ?? [], shownFolder),
					path
				)
			});
		},
		[activeTabId, getExplorerState, updateExplorerState, shownFolder]
	);

	const {table} = useTable(files, availableWidth, expansion);
	const {rows: sortedRows} = table.getRowModel();
	// Sections follow the column the list is sorted by.
	const sortColumn = table
		.getState()
		.sorting.find((entry) => entry.id !== 'folders')?.id;
	const groupBy = viewSettings.groupItems && isGroupBy(sortColumn) ? sortColumn : null;
	// Expanded folders' contents stay with the row that holds them.
	const blocks = useMemo(() => blocksByTopLevel(sortedRows), [sortedRows]);
	const groups = useMemo(
		() =>
			groupBy
				? groupFiles(
						sortedRows.filter((row) => row.depth === 0).map((row) => row.original),
						groupBy
					)
				: null,
		[sortedRows, groupBy]
	);
	// Sections reorder the rows, and selection ranges follow what is on screen.
	const rows = useMemo(() => {
		if (!groups) return sortedRows;
		return groups.flatMap((group) => group.files.flatMap((file) => blocks.get(file.id) ?? []));
	}, [groups, sortedRows, blocks]);
	const layout = useMemo(
		() =>
			buildListLayout(
				groups,
				rows.length,
				groups?.map((group) =>
					group.files.reduce((sum, file) => sum + (blocks.get(file.id)?.length ?? 1), 0)
				)
			),
		[groups, rows.length, blocks]
	);
	const orderedFiles = useMemo(() => rows.map((row) => row.original), [rows]);
	const columnSizingKey = JSON.stringify(table.getState().columnSizing);

	// Publishing the collection also reconciles the selection against it.
	useEffect(() => {
		setCurrentFiles(orderedFiles, unfilteredCount, files);
	}, [orderedFiles, unfilteredCount, files, setCurrentFiles]);

	// Virtual row rendering - uses the container as scroll element
	const rowVirtualizer = useVirtualizer({
		count: layout.items.length,
		initialOffset: scrollPosition.top,
		getScrollElement: useCallback(() => containerRef.current, []),
		estimateSize: useCallback(
			(index: number) =>
				layout.items[index]?.type === 'header' ? GROUP_HEADER_HEIGHT : ROW_HEIGHT,
			[layout]
		),
		paddingStart: TABLE_HEADER_HEIGHT + TABLE_PADDING_Y,
		paddingEnd: TABLE_PADDING_Y,
		overscan: 15
	});
	useTabScroll(containerRef, rows.length);

	// Item sizes are cached by index; headers move them when sections change.
	useEffect(() => {
		rowVirtualizer.measure();
	}, [layout, rowVirtualizer]);

	const virtualRows = rowVirtualizer.getVirtualItems();

	// Sync horizontal scroll between header and body
	const handleBodyScroll = useCallback(() => {
		if (bodyScrollRef.current && headerScrollRef.current) {
			headerScrollRef.current.scrollLeft =
				bodyScrollRef.current.scrollLeft;
		}
	}, []);

	// Inner wrappers fill the empty area, so anything that is not an item counts.
	const handleContainerContextMenu = async (e: React.MouseEvent) => {
		if ((e.target as Element).closest('[data-file-id]')) return;
		e.preventDefault();
		e.stopPropagation();
		await emptySpaceContextMenu.show(e);
	};

	// Store values in refs to avoid effect re-runs
	const rowVirtualizerRef = useRef(rowVirtualizer);
	rowVirtualizerRef.current = rowVirtualizer;
	const filesRef = useRef(orderedFiles);
	filesRef.current = orderedFiles;
	const rowsRef = useRef(rows);
	rowsRef.current = rows;
	const toggleExpandedRef = useRef(toggleExpanded);
	toggleExpandedRef.current = toggleExpanded;
	const itemOfFileRef = useRef(layout.itemOfFile);
	itemOfFileRef.current = layout.itemOfFile;

	// Rows are virtualized, so a revealed file may not be in the DOM yet.
	useEffect(() => {
		const handleReveal = (event: Event) => {
			const {fileId} = (event as CustomEvent<RevealDetail>).detail;
			const index = filesRef.current.findIndex((file) => file.id === fileId);
			if (index >= 0) {
				rowVirtualizerRef.current.scrollToIndex(itemOfFileRef.current[index] ?? index, {
					align: 'center'
				});
			}
		};
		window.addEventListener(REVEAL_EVENT, handleReveal);
		return () => window.removeEventListener(REVEAL_EVENT, handleReveal);
	}, []);

	// Keyboard navigation - stable effect, uses refs for changing values
	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if (isInputFocused()) return;
			// Right opens the focused folder in place and Left closes it, as in Dolphin.
			if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
				const row = rowsRef.current[focusedIndex];
				if (!row?.getCanExpand()) return;
				if (row.getIsExpanded() === (e.key === 'ArrowRight')) return;
				e.preventDefault();
				toggleExpandedRef.current(row.original);
				return;
			}
			if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
				e.preventDefault();
				const direction = e.key === 'ArrowDown' ? 'down' : 'up';
				const currentFiles = filesRef.current;

				const currentIndex = focusedIndex >= 0 ? focusedIndex : 0;
				const newIndex =
					direction === 'down'
						? Math.min(currentIndex + 1, currentFiles.length - 1)
						: Math.max(currentIndex - 1, 0);

				if (e.shiftKey) {
					// Range selection with shift
					if (newIndex !== focusedIndex && currentFiles[newIndex]) {
						selectFile(
							currentFiles[newIndex],
							currentFiles,
							false,
							true
						);
						setFocusedIndex(newIndex);
					}
				} else {
					moveFocus(direction, currentFiles);
				}

				// Scroll to keep selection visible
				rowVirtualizerRef.current.scrollToIndex(
					itemOfFileRef.current[newIndex] ?? newIndex,
					{align: 'auto'}
				);
			}
		};

		window.addEventListener('keydown', handleKeyDown);
		return () => window.removeEventListener('keydown', handleKeyDown);
	}, [focusedIndex, selectFile, setFocusedIndex, moveFocus]);

	// Calculate total width for table
	const headerGroups = table.getHeaderGroups();
	const totalWidth = Math.max(
		table.getTotalSize() + TABLE_PADDING_X * 2,
		availableWidth
	);

	return (
		<div
			ref={containerRef}
			className="h-full overflow-auto"
			onContextMenu={handleContainerContextMenu}
		>
			<DragSelect files={orderedFiles} scrollRef={containerRef}>
				{/* Sticky Header */}
				<div
					className="border-app-line bg-app/90 sticky top-0 z-10 border-b backdrop-blur-lg"
					style={{height: TABLE_HEADER_HEIGHT}}
				>
					<div ref={headerScrollRef} className="overflow-hidden">
						<div
							className="flex"
							style={{
								width: totalWidth,
								paddingLeft: TABLE_PADDING_X,
								paddingRight: TABLE_PADDING_X
							}}
						>
							{headerGroups.map((headerGroup) =>
								headerGroup.headers.map((header) => {
									const isSorted =
										header.column.getIsSorted();
									const canResize =
										header.column.getCanResize();

									return (
										<div
											key={header.id}
											role="columnheader"
											tabIndex={0}
											aria-sort={
												isSorted === 'asc'
													? 'ascending'
													: isSorted === 'desc'
														? 'descending'
														: 'none'
											}
											onKeyDown={(event) => {
												if (
													event.key === 'Enter' ||
													event.key === ' '
												) {
													event.preventDefault();
													header.column.toggleSorting(
														undefined,
														event.shiftKey
													);
												}
											}}
											className={clsx(
												'relative flex items-center gap-1 px-2 py-2 text-xs font-medium select-none',
												isSorted
													? 'text-ink'
													: 'text-ink-dull',
												'hover:text-ink cursor-pointer'
											)}
											style={{width: header.getSize()}}
											onClick={header.column.getToggleSortingHandler()}
										>
											<span className="truncate">
								{typeof header.column.columnDef.header === 'string'
									? t(header.column.columnDef.header)
									: flexRender(header.column.columnDef.header, header.getContext())}
											</span>

											{isSorted &&
												(isSorted === 'asc' ? (
													<CaretUp className="text-ink-faint size-3 flex-shrink-0" />
												) : (
													<CaretDown className="text-ink-faint size-3 flex-shrink-0" />
												))}

											{/* Resize handle; double-click restores the default width */}
											{canResize && (
												<div
													onMouseDown={header.getResizeHandler()}
													onTouchStart={header.getResizeHandler()}
													onClick={(e) =>
														e.stopPropagation()
													}
													onDoubleClick={(e) => {
														e.stopPropagation();
														header.column.resetSize();
													}}
													className="group/resize absolute inset-y-0 -right-1.5 z-10 flex w-3 cursor-col-resize items-center justify-center"
												>
													<div
														className={clsx(
															'h-4 w-0.5 rounded-full',
															header.column.getIsResizing()
																? 'bg-accent'
																: 'group-hover/resize:bg-ink-faint/50 bg-transparent'
														)}
													/>
												</div>
											)}
										</div>
									);
								})
							)}
						</div>
					</div>
				</div>

				{/* Virtual List Body */}
				<div
					ref={bodyScrollRef}
					className="overflow-x-auto"
					onScroll={handleBodyScroll}
					style={{pointerEvents: 'auto'}}
				>
					<div
						className="relative"
						style={{
							height:
								rowVirtualizer.getTotalSize() -
								TABLE_HEADER_HEIGHT,
							width: totalWidth,
							pointerEvents: 'auto'
						}}
					>
						<div
							className="absolute top-0 left-0 w-full"
							style={{
								transform: `translateY(${(virtualRows[0]?.start ?? 0) - TABLE_HEADER_HEIGHT - TABLE_PADDING_Y}px)`,
								pointerEvents: 'auto'
							}}
						>
							{virtualRows.map((virtualRow) => {
								const item = layout.items[virtualRow.index];
								if (!item) return null;

								if (item.type === 'header') {
									return (
										<div
											key={`group:${item.group.key}`}
											ref={rowVirtualizer.measureElement}
											data-index={virtualRow.index}
											style={{
												paddingLeft: TABLE_PADDING_X + 8,
												paddingRight: TABLE_PADDING_X + 8
											}}
										>
											<GroupHeader
												label={groupLabel(item.group.label)}
												count={item.group.files.length}
											/>
										</div>
									);
								}

								const row = rows[item.index];
								if (!row) return null;

								const file = row.original;
								// Rows inside an expanded folder take that folder's Git state.
								const parentRow = row.getParentRow();
								const parentPath = parentRow ? physicalPathOf(parentRow.original) : null;
								const folderVcsStatus = parentRow
									? parentPath
										? vcsStatusByFolder.get(parentPath)
										: undefined
									: vcsStatus;
								// Use O(1) lookup instead of O(n) selectedFiles.some()
								const fileIsSelected = isSelected(file.id);
								const isFocused = focusedIndex === item.index;
								// A header between two rows breaks the selection band.
								const previousItem = layout.items[virtualRow.index - 1];
								const nextItem = layout.items[virtualRow.index + 1];
								const previousRow =
									previousItem?.type === 'file' ? rows[previousItem.index] : undefined;
								const nextRow =
									nextItem?.type === 'file' ? rows[nextItem.index] : undefined;
								// Use O(1) Set lookup for adjacent selection detection
								const isPreviousSelected = previousRow
									? selectedFileIds.has(
											previousRow.original.id
										)
									: false;
								const isNextSelected = nextRow
									? selectedFileIds.has(nextRow.original.id)
									: false;

								return (
									<TableRow
										columnSizingKey={columnSizingKey}
										key={row.id}
										row={row}
										file={file}
										files={orderedFiles}
										index={item.index}
										virtualIndex={virtualRow.index}
										isSelected={fileIsSelected}
										isFocused={isFocused}
										isPreviousSelected={isPreviousSelected}
										isNextSelected={isNextSelected}
										measureRef={
											rowVirtualizer.measureElement
										}
										selectedFiles={
											fileIsSelected ? selectedFiles : NO_FILES
										}
										rename={
											renamingFileId === file.id
												? rename
												: undefined
										}
										vcsStatus={vcsStatusOf(folderVcsStatus, file)}
										onToggleExpanded={toggleExpanded}
										selectFile={selectFile}
									/>
								);
							})}
						</div>
					</div>
				</div>
			</DragSelect>
		</div>
	);
});
