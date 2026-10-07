import {CaretDown, CaretUp} from '@phosphor-icons/react';
import {flexRender} from '@tanstack/react-table';
import {useVirtualizer} from '@tanstack/react-virtual';
import clsx from 'clsx';
import {memo, useCallback, useEffect, useMemo, useRef} from 'react';
import {isInputFocused} from '../../../../util/keybinds/platform';
import {useExplorer} from '../../context';
import {useEmptySpaceContextMenu} from '../../hooks/useEmptySpaceContextMenu';
import {useExplorerFiles} from '../../hooks/useExplorerFiles';
import {REVEAL_EVENT, type RevealDetail} from '../../pendingReveal';
import {useTabScroll} from '../../hooks/useTabScroll';
import {useSelection} from '../../SelectionContext';
import {DragSelect} from './DragSelect';
import {TableRow} from './TableRow';
import {
	ROW_HEIGHT,
	TABLE_HEADER_HEIGHT,
	TABLE_PADDING_X,
	TABLE_PADDING_Y,
	useTable
} from './useTable';

export const ListView = memo(function ListView() {
	const {setCurrentFiles, scrollPosition} = useExplorer();
	const {
		focusedIndex,
		setFocusedIndex,
		selectedFileIds,
		isSelected,
		selectFile,
		moveFocus
	} = useSelection();

	const containerRef = useRef<HTMLDivElement>(null);
	const headerScrollRef = useRef<HTMLDivElement>(null);
	const bodyScrollRef = useRef<HTMLDivElement>(null);
	const emptySpaceContextMenu = useEmptySpaceContextMenu();

	// Get files from centralized hook (handles search, virtual, and directory)
	const {files} = useExplorerFiles();
	const {table} = useTable(files);
	const {rows} = table.getRowModel();
	const orderedFiles = useMemo(() => rows.map((row) => row.original), [rows]);
	const columnSizingKey = JSON.stringify(table.getState().columnSizing);

	// Publishing the collection also reconciles the selection against it.
	useEffect(() => {
		setCurrentFiles(orderedFiles);
	}, [orderedFiles, setCurrentFiles]);

	// Virtual row rendering - uses the container as scroll element
	const rowVirtualizer = useVirtualizer({
		count: rows.length,
		initialOffset: scrollPosition.top,
		getScrollElement: useCallback(() => containerRef.current, []),
		estimateSize: useCallback(() => ROW_HEIGHT, []),
		paddingStart: TABLE_HEADER_HEIGHT + TABLE_PADDING_Y,
		paddingEnd: TABLE_PADDING_Y,
		overscan: 15
	});
	useTabScroll(containerRef, rows.length);

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

	// Rows are virtualized, so a revealed file may not be in the DOM yet.
	useEffect(() => {
		const handleReveal = (event: Event) => {
			const {fileId} = (event as CustomEvent<RevealDetail>).detail;
			const index = filesRef.current.findIndex((file) => file.id === fileId);
			if (index >= 0) rowVirtualizerRef.current.scrollToIndex(index, {align: 'center'});
		};
		window.addEventListener(REVEAL_EVENT, handleReveal);
		return () => window.removeEventListener(REVEAL_EVENT, handleReveal);
	}, []);

	// Keyboard navigation - stable effect, uses refs for changing values
	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if (isInputFocused()) return;
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
				rowVirtualizerRef.current.scrollToIndex(newIndex, {
					align: 'auto'
				});
			}
		};

		window.addEventListener('keydown', handleKeyDown);
		return () => window.removeEventListener('keydown', handleKeyDown);
	}, [focusedIndex, selectFile, setFocusedIndex, moveFocus]);

	// Calculate total width for table
	const headerGroups = table.getHeaderGroups();
	const totalWidth = table.getTotalSize() + TABLE_PADDING_X * 2;

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
												{flexRender(
													header.column.columnDef
														.header,
													header.getContext()
												)}
											</span>

											{isSorted &&
												(isSorted === 'asc' ? (
													<CaretUp className="text-ink-faint size-3 flex-shrink-0" />
												) : (
													<CaretDown className="text-ink-faint size-3 flex-shrink-0" />
												))}

											{/* Resize handle */}
											{canResize && (
												<div
													onMouseDown={header.getResizeHandler()}
													onTouchStart={header.getResizeHandler()}
													onClick={(e) =>
														e.stopPropagation()
													}
													className={clsx(
														'absolute top-1/2 right-0 h-4 w-1 -translate-y-1/2 cursor-col-resize rounded-full',
														header.column.getIsResizing()
															? 'bg-accent'
															: 'hover:bg-ink-faint/50 bg-transparent'
													)}
												/>
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
								const row = rows[virtualRow.index];
								if (!row) return null;

								const file = row.original;
								// Use O(1) lookup instead of O(n) selectedFiles.some()
								const fileIsSelected = isSelected(file.id);
								const isFocused =
									focusedIndex === virtualRow.index;
								const previousRow = rows[virtualRow.index - 1];
								const nextRow = rows[virtualRow.index + 1];
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
										index={virtualRow.index}
										isSelected={fileIsSelected}
										isFocused={isFocused}
										isPreviousSelected={isPreviousSelected}
										isNextSelected={isNextSelected}
										measureRef={
											rowVirtualizer.measureElement
										}
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
