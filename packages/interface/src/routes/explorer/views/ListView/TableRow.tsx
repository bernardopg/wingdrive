import {useDroppable} from '@dnd-kit/core';
import {flexRender} from '@tanstack/react-table';
import type {LegacyRow as Row} from '@tanstack/react-table/legacy';
import type {File} from '@wingdrive/ts-client';
import clsx from 'clsx';
import {memo, useCallback} from 'react';
import {TagPill} from '../../../../components/Tags';
import {InlineNameEdit} from '../../components/InlineNameEdit';
import {File as FileComponent} from '../../File';
import {useDraggableFile} from '../../hooks/useDraggableFile';
import {useFileContextMenu} from '../../hooks/useFileContextMenu';
import {useOpenFile} from '../../hooks/useOpenFile';
import {useSelection} from '../../SelectionContext';
import {ROW_HEIGHT, TABLE_PADDING_X} from './useTable';

interface TableRowProps {
	columnSizingKey: string;
	row: Row<File>;
	file: File;
	files: File[];
	index: number;
	isSelected: boolean;
	isFocused: boolean;
	isPreviousSelected: boolean;
	isNextSelected: boolean;
	measureRef: (node: HTMLElement | null) => void;
	selectFile: (
		file: File,
		files: File[],
		multi?: boolean,
		range?: boolean
	) => void;
}

export const TableRow = memo(function TableRow({
	row,
	file,
	files,
	index,
	isSelected,
	isFocused: _isFocused,
	isPreviousSelected,
	isNextSelected,
	measureRef,
	selectFile
}: TableRowProps) {
	const {selectedFiles} = useSelection();

	const contextMenu = useFileContextMenu({
		file,
		selectedFiles,
		selected: isSelected
	});

	const openFile = useOpenFile();

	const handleClick = useCallback(
		(e: React.MouseEvent) => {
			const multi = e.metaKey || e.ctrlKey;
			const range = e.shiftKey;
			selectFile(file, files, multi, range);
		},
		[file, files, selectFile]
	);

	const handleDoubleClick = () => openFile(file);

	const handleContextMenu = useCallback(
		async (e: React.MouseEvent) => {
			e.preventDefault();
			e.stopPropagation();

			if (!isSelected) {
				selectFile(file, files, false, false);
			}

			await contextMenu.show(e);
		},
		[file, files, isSelected, selectFile, contextMenu]
	);

	const cells = row.getVisibleCells();

	// List view previously had no drag source or drop target at all: rows
	// could neither start a drag nor receive one, so rearranging files was
	// only possible in the grid view.
	const {
		attributes,
		listeners,
		setNodeRef: setDragNodeRef,
		isDragging
	} = useDraggableFile({
		file,
		selectedFiles:
			isSelected && selectedFiles.length > 0 ? selectedFiles : undefined
	});

	const isFolder = file.kind === 'Directory';
	const {setNodeRef: setDropNodeRef, isOver: isDropOver} = useDroppable({
		id: `folder-drop-${file.id}`,
		disabled: !isFolder,
		data: {
			action: 'move-into',
			targetType: 'folder',
			targetId: file.id,
			targetPath: file.wing_path
		}
	});

	const setCombinedNodeRef = useCallback(
		(node: HTMLElement | null) => {
			measureRef(node);
			setDragNodeRef(node);
			if (isFolder) setDropNodeRef(node);
		},
		[measureRef, setDragNodeRef, setDropNodeRef, isFolder]
	);

	return (
		<div
			ref={setCombinedNodeRef}
			data-index={index}
			data-file-id={file.id}
			data-selectable="true"
			aria-selected={isSelected}
			className={clsx(
				'relative outline-none focus:outline-none',
				isDragging && 'opacity-40'
			)}
			style={{height: ROW_HEIGHT}}
			onClick={handleClick}
			onDoubleClick={handleDoubleClick}
			onContextMenu={handleContextMenu}
			{...attributes}
			{...listeners}
			tabIndex={-1}
		>
			{/* Background layer for alternating colors and selection */}
			<div
				className={clsx(
					'absolute inset-0 rounded-md border',
					// Alternating background
					index % 2 === 0 && !isSelected && 'bg-app-dark-box/50',
					// Selection styling
					isSelected
						? 'border-accent bg-accent/10'
						: 'border-transparent',
					// Connect adjacent selected rows
					isSelected &&
						isPreviousSelected &&
						'rounded-t-none border-t-0',
					isSelected && isNextSelected && 'rounded-b-none border-b-0',
					// Drop indicator for folders
					isFolder &&
						isDropOver &&
						!isSelected &&
						'border-accent/60 bg-accent/5'
				)}
				style={{
					left: TABLE_PADDING_X,
					right: TABLE_PADDING_X
				}}
			>
				{/* Subtle separator between connected selected rows */}
				{isSelected && isPreviousSelected && (
					<div className="bg-accent/20 absolute inset-x-3 top-0 h-px" />
				)}
			</div>

			{/* Row content */}
			<div
				className="relative flex h-full items-center"
				style={{
					paddingLeft: TABLE_PADDING_X,
					paddingRight: TABLE_PADDING_X
				}}
			>
				{cells.map((cell) => {
					const isNameColumn = cell.column.id === 'name';

					return (
						<div
							key={cell.id}
							className={clsx(
								'flex h-full items-center px-2 text-sm',
								isNameColumn ? 'min-w-0' : 'text-ink-dull'
							)}
							style={{width: cell.column.getSize()}}
						>
							{isNameColumn ? (
								<NameCell file={file} />
							) : (
								<span className="truncate">
									{flexRender(
										cell.column.columnDef.cell,
										cell.getContext()
									)}
								</span>
							)}
						</div>
					);
				})}
			</div>
		</div>
	);
});

// Name cell with icon and tags
const NameCell = memo(function NameCell({file}: {file: File}) {
	const {renamingFileId, saveRename, cancelRename} = useSelection();
	const isRenaming = renamingFileId === file.id;

	return (
		<div className="flex min-w-0 flex-1 items-center gap-2">
			{/* File icon */}
			<div className="flex-shrink-0">
				<FileComponent.Thumb file={file} size={20} />
			</div>

			{/* File name or inline edit */}
			{isRenaming ? (
				<InlineNameEdit
					file={file}
					onSave={saveRename}
					onCancel={cancelRename}
					className="min-w-0 flex-1"
				/>
			) : (
				<span className="text-ink truncate text-sm">
					{file.name}
					{file.extension && `.${file.extension}`}
				</span>
			)}

			{/* Tags (inline, compact) - hide when renaming */}
			{!isRenaming && file.tags && file.tags.length > 0 && (
				<div className="flex flex-shrink-0 items-center gap-1">
					{file.tags.slice(0, 2).map((tag) => (
						<TagPill
							key={tag.id}
							color={tag.color || '#3B82F6'}
							size="xs"
						>
							{tag.canonical_name}
						</TagPill>
					))}
					{file.tags.length > 2 && (
						<span className="text-ink-faint text-[10px]">
							+{file.tags.length - 2}
						</span>
					)}
				</div>
			)}
		</div>
	);
});
