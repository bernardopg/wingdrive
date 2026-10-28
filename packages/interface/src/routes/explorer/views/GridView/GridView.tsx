import {useVirtualizer} from '@tanstack/react-virtual';
import type {File, VcsStatusOutput} from '@wingdrive/ts-client';
import {useEffect, useMemo, useRef, useState} from 'react';
import {isInputFocused} from '../../../../util/keybinds/platform';
import {vcsStatusOf} from '../../components/vcsStatus';
import {useExplorer} from '../../context';
import {useEmptySpaceContextMenu} from '../../hooks/useEmptySpaceContextMenu';
import {useExplorerFiles} from '../../hooks/useExplorerFiles';
import {useFolderFilter} from '../../hooks/useFolderFilter';
import {REVEAL_EVENT, type RevealDetail} from '../../pendingReveal';
import {useTabScroll} from '../../hooks/useTabScroll';
import {useSelection} from '../../SelectionContext';
import {DragSelect} from './DragSelect';
import {FileCard} from './FileCard';
import {useTranslation} from '../../../../i18n';
import {GROUP_HEADER_HEIGHT, GroupHeader, useGroupLabel} from '../GroupHeader';
import {
	buildGridLayout,
	gridNeighbor,
	groupFiles,
	isGroupBy,
	type FileGroup
} from '../grouping';

const VIRTUALIZATION_THRESHOLD = 0; // Disabled - always virtualize
let lastGridMeasurement: {key: string; width: number; height: number} | null = null;

const NO_FILES: File[] = [];

/**
 * Props that change only for the cards they concern, so a selection or
 * rename does not re-render every card in the grid.
 */
function useCardProps(vcsStatus: VcsStatusOutput | undefined) {
	const {viewSettings} = useExplorer();
	const {selectedFiles, renamingFileId, saveRename, cancelRename} =
		useSelection();
	const rename = useMemo(
		() => ({onSave: saveRename, onCancel: cancelRename}),
		[saveRename, cancelRename]
	);
	return (file: File, selected: boolean) => ({
		gridSize: viewSettings.gridSize,
		showFileSize: viewSettings.showFileSize,
		selectedFiles: selected ? selectedFiles : NO_FILES,
		rename: renamingFileId === file.id ? rename : undefined,
		vcsStatus: vcsStatusOf(vcsStatus, file)
	});
}

export function GridView() {
	const {t} = useTranslation('explorer');
	const {viewSettings, sortBy, setCurrentFiles} = useExplorer();
	const {
		isSelected,
		focusedIndex,
		setFocusedIndex,
		selectFile,
		clearSelection,
		setSelectedFiles
	} = useSelection();
	const {gridSize, gapSize} = viewSettings;
	const emptySpaceContextMenu = useEmptySpaceContextMenu();

	// Get files from centralized hook (handles search, virtual, and directory)
	const {files: listedFiles, isLoading, source, vcsStatus} = useExplorerFiles();
	const cardProps = useCardProps(vcsStatus);
	const {files: shownFiles, unfilteredCount} = useFolderFilter(listedFiles);
	const groupBy = viewSettings.groupItems && isGroupBy(sortBy) ? sortBy : null;
	const groups = useMemo(
		() => (groupBy ? groupFiles(shownFiles, groupBy) : null),
		[shownFiles, groupBy]
	);
	// Sections reorder the listing, and selection ranges follow what is on screen.
	const files = useMemo(
		() => (groups ? groups.flatMap((group) => group.files) : shownFiles),
		[groups, shownFiles]
	);

	// Publishing the collection also reconciles the selection against it.
	useEffect(() => {
		setCurrentFiles(files, unfilteredCount);
	}, [files, unfilteredCount, setCurrentFiles]);

	const handleContainerClick = (e: React.MouseEvent) => {
		if (e.target === e.currentTarget) {
			clearSelection();
		}
	};

	// Inner wrappers fill the empty area, so anything that is not an item counts.
	const handleContainerContextMenu = async (e: React.MouseEvent) => {
		if ((e.target as Element).closest('[data-file-id]')) return;
		e.preventDefault();
		e.stopPropagation();
		await emptySpaceContextMenu.show(e);
	};

	// Conditional virtualization - use simple grid for small directories
	const shouldVirtualize = files.length > VIRTUALIZATION_THRESHOLD;
	const gridContainerRef = useRef<HTMLDivElement>(null);

	// Empty state for tag view with no tagged files
	if (source === 'tag' && files.length === 0 && !isLoading) {
		return (
			<div className="flex h-full items-center justify-center">
				<div className="text-center">
					<div className="text-ink-dull mb-1 text-lg font-medium">
						{t("ui.noTaggedFiles")}
					</div>
					<div className="text-ink-dull text-sm">
						{t("ui.filesTaggedWithThisTagWill")}
					</div>
				</div>
			</div>
		);
	}

	if (!shouldVirtualize) {
		return (
			<div
				ref={gridContainerRef}
				className="h-full overflow-auto"
				onClick={handleContainerClick}
				onContextMenu={handleContainerContextMenu}
			>
				<DragSelect files={files} scrollRef={gridContainerRef}>
					<div
						className="grid min-h-full p-3"
						style={{
							gridTemplateColumns: `repeat(auto-fill, minmax(${gridSize}px, 1fr))`,
							gridAutoRows: 'max-content',
							gap: `${gapSize}px`
						}}
					>
						{files.map((file, index) => (
							<FileCard
								key={file.id}
								file={file}
								fileIndex={index}
								allFiles={files}
								selected={isSelected(file.id)}
								focused={index === focusedIndex}
								selectFile={selectFile}
								{...cardProps(file, isSelected(file.id))}
							/>
						))}
					</div>
				</DragSelect>
			</div>
		);
	}

	return (
		<VirtualizedGrid
			files={files}
			gridSize={gridSize}
			gapSize={gapSize}
			isSelected={isSelected}
			focusedIndex={focusedIndex}
			setFocusedIndex={setFocusedIndex}
			selectFile={selectFile}
			setSelectedFiles={setSelectedFiles}
			onContainerClick={handleContainerClick}
			onContainerContextMenu={handleContainerContextMenu}
			vcsStatus={vcsStatus}
			groups={groups}
		/>
	);
}

interface VirtualizedGridProps {
	files: File[];
	gridSize: number;
	gapSize: number;
	isSelected: (id: string) => boolean;
	focusedIndex: number;
	setFocusedIndex: (index: number) => void;
	selectFile: (
		file: File,
		files: File[],
		multi?: boolean,
		range?: boolean
	) => void;
	setSelectedFiles: (files: File[]) => void;
	onContainerClick: (e: React.MouseEvent) => void;
	onContainerContextMenu: (e: React.MouseEvent) => void;
	vcsStatus: VcsStatusOutput | undefined;
	/** Sections to draw, or `null` for a plain grid. */
	groups: FileGroup[] | null;
}

function VirtualizedGrid({
	files,
	gridSize,
	gapSize,
	isSelected,
	focusedIndex,
	setFocusedIndex,
	selectFile,
	setSelectedFiles,
	onContainerClick,
	onContainerContextMenu,
	vcsStatus,
	groups
}: VirtualizedGridProps) {
	const cardProps = useCardProps(vcsStatus);
	const groupLabel = useGroupLabel();
	const {sidebarVisible, inspectorVisible, scrollPosition} = useExplorer();
	const measurementKey = `${window.innerWidth}:${window.innerHeight}:${sidebarVisible}:${inspectorVisible}`;
	const measurementKeyRef = useRef(measurementKey);
	measurementKeyRef.current = measurementKey;
	const parentRef = useRef<HTMLDivElement>(null);
	const [containerWidth, setContainerWidth] = useState<number | null>(() => lastGridMeasurement?.key === measurementKey ? lastGridMeasurement.width : null);
	const [isInitialized, setIsInitialized] = useState(containerWidth !== null);

	// Calculate columns (mimic auto-fill behavior)
	const columns = useMemo(() => {
		if (!containerWidth) return 1;

		// Mimic repeat(auto-fill, minmax(gridSize, 1fr))
		const minItemWidth = gridSize;

		// Calculate how many items fit
		let cols = 1;
		while (true) {
			const totalGaps = (cols - 1) * gapSize;
			const requiredWidth = cols * minItemWidth + totalGaps;

			if (requiredWidth <= containerWidth) {
				cols++;
			} else {
				cols--;
				break;
			}
		}

		return Math.max(1, cols);
	}, [containerWidth, gridSize, gapSize]);

	const layout = useMemo(
		() => buildGridLayout(groups, files, columns),
		[groups, files, columns]
	);
	const rowGap = 4; // Gap between rows

	// Row virtualizer
	const rowVirtualizer = useVirtualizer({
		count: layout.rows.length,
		initialOffset: scrollPosition.top,
		initialRect: {width: containerWidth ?? 0, height: lastGridMeasurement?.key === measurementKey ? lastGridMeasurement.height : 0},
		getScrollElement: () => parentRef.current,
		observeElementRect(instance, callback) {
			if (!instance.scrollElement) return;
			if (lastGridMeasurement?.key === measurementKeyRef.current) {
				callback({width: lastGridMeasurement.width + 24, height: lastGridMeasurement.height});
			}
			// One observer feeds both the grid and virtualizer without forcing layout.
			const observer = new ResizeObserver(([entry]) => {
				const width = entry.borderBoxSize[0]?.inlineSize ?? entry.contentRect.width;
				const height = entry.borderBoxSize[0]?.blockSize ?? entry.contentRect.height;
				if (width <= 0) return;
				lastGridMeasurement = {key: measurementKeyRef.current, width: width - 24, height};
				setContainerWidth(width - 24);
				setIsInitialized(true);
				callback({width: Math.round(width), height: Math.round(height)});
			});
			observer.observe(instance.scrollElement);
			return () => observer.disconnect();
		},
		estimateSize: (index) =>
			layout.rows[index]?.type === 'header'
				? GROUP_HEADER_HEIGHT
				: gridSize + gapSize + rowGap,
		overscan: 5
	});
	useTabScroll(parentRef, isInitialized ? files.length : 0);

	// Row sizes are cached by index; headers move them when sections change.
	useEffect(() => {
		rowVirtualizer.measure();
	}, [layout, rowVirtualizer]);

	const virtualRows = rowVirtualizer.getVirtualItems();

	// Rows are virtualized, so scroll the revealed file's row into existence.
	// The reveal can arrive before the grid has measured its columns, so it
	// waits in a ref until the layout is ready.
	const pendingRevealRef = useRef<string | null>(null);
	const applyRevealRef = useRef(() => {});
	applyRevealRef.current = () => {
		const fileId = pendingRevealRef.current;
		if (!fileId || !isInitialized) return;
		const index = files.findIndex((file) => file.id === fileId);
		if (index < 0) return;
		pendingRevealRef.current = null;
		rowVirtualizer.scrollToIndex(layout.rowOfFile[index] ?? 0, {align: 'center'});
	};
	useEffect(() => {
		const handleReveal = (event: Event) => {
			pendingRevealRef.current = (event as CustomEvent<RevealDetail>).detail.fileId;
			applyRevealRef.current();
		};
		window.addEventListener(REVEAL_EVENT, handleReveal);
		return () => window.removeEventListener(REVEAL_EVENT, handleReveal);
	}, []);
	useEffect(() => {
		applyRevealRef.current();
	}, [files, layout, isInitialized]);

	// Keyboard navigation with correct column count
	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if (isInputFocused()) return;
			if (
				!['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(
					e.key
				)
			) {
				return;
			}
			if (files.length === 0) return;

			e.preventDefault();

			let newIndex = focusedIndex < 0 ? 0 : focusedIndex;

			if (e.key === 'ArrowUp') {
				newIndex = gridNeighbor(layout, newIndex, 'up');
			} else if (e.key === 'ArrowDown') {
				newIndex = gridNeighbor(layout, newIndex, 'down');
			} else if (e.key === 'ArrowLeft') {
				newIndex = Math.max(0, newIndex - 1);
			} else if (e.key === 'ArrowRight') {
				newIndex = Math.min(files.length - 1, newIndex + 1);
			}

			if (newIndex !== focusedIndex && files[newIndex]) {
				setFocusedIndex(newIndex);
				setSelectedFiles([files[newIndex]]);

				// Scroll into view
				const element = document.querySelector(
					`[data-file-id="${files[newIndex].id}"]`
				);
				if (element) {
					element.scrollIntoView({
						block: 'nearest',
						behavior: 'smooth'
					});
				}
			}
		};

		window.addEventListener('keydown', handleKeyDown);
		return () => window.removeEventListener('keydown', handleKeyDown);
	}, [files, focusedIndex, layout, setFocusedIndex, setSelectedFiles]);

	return (
		<div
			ref={parentRef}
			className="h-full overflow-auto"
			onClick={onContainerClick}
			onContextMenu={onContainerContextMenu}
		>
			<DragSelect files={files} scrollRef={parentRef}>
				<div
					className="relative"
					style={{
						height: `${rowVirtualizer.getTotalSize()}px`,
						paddingTop: '12px',
						paddingBottom: '12px',
						minHeight: '100%',
						opacity: isInitialized ? 1 : 0,
						transition: 'opacity 0.1s'
					}}
				>
					{virtualRows.map((virtualRow) => {
						const row = layout.rows[virtualRow.index];
						if (!row) return null;

						if (row.type === 'header') {
							return (
								<GroupHeader
									key={virtualRow.key}
									label={groupLabel(row.group.label)}
									count={row.group.files.length}
									className="absolute left-0 w-full px-3"
									style={{top: `${virtualRow.start}px`}}
								/>
							);
						}

						const startIndex = row.start;
						const rowFiles = row.files;

						return (
							<div
								key={virtualRow.key}
								className="absolute left-0 w-full px-3"
								style={{
									top: `${virtualRow.start}px`,
									height: `${gridSize + gapSize}px`
								}}
							>
								{/* CSS Grid within row - preserves flex-to-fill */}
								<div
									className="grid h-full"
									style={{
										gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
										gap: `${gapSize}px`
									}}
								>
									{rowFiles.map((file, idx) => {
										const fileIndex = startIndex + idx;
										return (
											<FileCard
												key={file.id}
												file={file}
												fileIndex={fileIndex}
												allFiles={files}
												selected={isSelected(file.id)}
												focused={
													fileIndex === focusedIndex
												}
												selectFile={selectFile}
												{...cardProps(
													file,
													isSelected(file.id)
												)}
											/>
										);
									})}
								</div>
							</div>
						);
					})}
				</div>
			</DragSelect>
		</div>
	);
}
