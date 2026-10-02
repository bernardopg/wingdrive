import {useVirtualizer, type VirtualItem} from '@tanstack/react-virtual';
import type {File, WingPath} from '@wingdrive/ts-client';
import clsx from 'clsx';
import {memo, useCallback, useRef} from 'react';
import {useNormalizedQuery} from '../../../../contexts/WingDriveContext';
import {useExplorer} from '../../context';
import {useFileContextMenu} from '../../hooks/useFileContextMenu';
import {useOpenFile} from '../../hooks/useOpenFile';
import {useSelection} from '../../SelectionContext';
import {ColumnItem} from './ColumnItem';

/**
 * Memoized wrapper for ColumnItem to prevent re-renders when selection changes elsewhere.
 * Only re-renders when this specific item's `selected` state changes.
 */
const ColumnItemWrapper = memo(function ColumnItemWrapper({
	file,
	files,
	virtualRow,
	selected,
	selectedFiles,
	onSelectFile,
	onNavigate
}: {
	file: File;
	files: File[];
	virtualRow: VirtualItem;
	selected: boolean;
	selectedFiles: File[];
	onSelectFile: (
		file: File,
		files: File[],
		multi?: boolean,
		range?: boolean
	) => void;
	onNavigate: (path: WingPath) => void;
}) {
	const contextMenu = useFileContextMenu({
		file,
		selectedFiles,
		selected
	});

	const handleClick = useCallback(
		(multi: boolean, range: boolean) => {
			onSelectFile(file, files, multi, range);
		},
		[file, files, onSelectFile]
	);

	const openFile = useOpenFile(onNavigate);
	const handleDoubleClick = () => openFile(file);

	const handleContextMenu = useCallback(
		async (e: React.MouseEvent) => {
			e.preventDefault();
			e.stopPropagation();
			if (!selected) {
				onSelectFile(file, files, false, false);
			}
			await contextMenu.show(e);
		},
		[file, files, selected, onSelectFile, contextMenu]
	);

	return (
		<div
			style={{
				position: 'absolute',
				top: 0,
				left: 0,
				width: '100%',
				height: `${virtualRow.size}px`,
				transform: `translateY(${virtualRow.start}px)`
			}}
		>
			<ColumnItem
				file={file}
				selected={selected}
				focused={false}
				onClick={handleClick}
				onDoubleClick={handleDoubleClick}
				onContextMenu={handleContextMenu}
			/>
		</div>
	);
});

interface ColumnProps {
	path: WingPath | null;
	isSelected: (fileId: string) => boolean;
	selectedFileIds: Set<string>;
	onSelectFile: (
		file: File,
		files: File[],
		multi?: boolean,
		range?: boolean
	) => void;
	onNavigate: (path: WingPath) => void;
	nextColumnPath?: WingPath;
	columnIndex: number;
	isActive: boolean;
	virtualFiles?: File[];
}

export const Column = memo(function Column({
	path,
	isSelected,
	onSelectFile,
	onNavigate,
	nextColumnPath,
	isActive,
	virtualFiles
}: ColumnProps) {
	const parentRef = useRef<HTMLDivElement>(null);
	const {viewSettings, sortBy, sortDirection} = useExplorer();
	const {selectedFiles} = useSelection();

	const directoryQuery = useNormalizedQuery({
		query: 'files.directory_listing',
		input: {
			path: path!,
			limit: null,
			include_hidden: viewSettings.showHiddenFiles,
			sort_by: sortBy as any,
			folders_first: viewSettings.foldersFirst,
			sort_direction: sortDirection
		},
		resourceType: 'file',
		pathScope: path ?? undefined,
		enabled: !!path && !virtualFiles
		// includeDescendants defaults to false for exact directory matching
	});

	const files = virtualFiles || (directoryQuery.data as any)?.files || [];

	const rowVirtualizer = useVirtualizer({
		count: files.length,
		getScrollElement: () => parentRef.current,
		estimateSize: () => 32,
		overscan: 10
	});

	// Only show loading state if we're not using virtual files and the query is actually loading
	if (!virtualFiles && directoryQuery.isLoading) {
		return (
			<div
				className="border-app-line flex shrink-0 items-center justify-center border-r"
				style={{width: `${viewSettings.columnWidth}px`}}
			>
				<div className="text-ink-dull text-sm">Loading...</div>
			</div>
		);
	}

	return (
		<div
			ref={parentRef}
			className={clsx(
				'border-app-line shrink-0 overflow-auto border-r',
				isActive && 'bg-app-box/30'
			)}
			style={{width: `${viewSettings.columnWidth}px`}}
		>
			<div
				style={{
					height: `${rowVirtualizer.getTotalSize()}px`,
					width: '100%',
					position: 'relative'
				}}
			>
				{rowVirtualizer.getVirtualItems().map((virtualRow) => {
					const file = files[virtualRow.index];

					// Check if this file is selected using O(1) lookup
					const fileIsSelected = isSelected(file.id);

					// Check if this file is part of the navigation path
					const isInPath =
						nextColumnPath && file.wing_path
							? JSON.stringify(file.wing_path) ===
								JSON.stringify(nextColumnPath)
							: false;

					return (
						<ColumnItemWrapper
							key={virtualRow.key}
							file={file}
							files={files}
							virtualRow={virtualRow}
							selected={fileIsSelected || isInPath}
							selectedFiles={selectedFiles}
							onSelectFile={onSelectFile}
							onNavigate={onNavigate}
						/>
					);
				})}
			</div>
		</div>
	);
});
