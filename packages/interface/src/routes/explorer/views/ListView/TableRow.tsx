import {useDroppable} from '@dnd-kit/core';
import {flexRender} from '@tanstack/react-table';
import type {LegacyRow as Row} from '@tanstack/react-table/legacy';
import type {File, VcsFileStatus} from '@wingdrive/ts-client';
import clsx from 'clsx';
import {CaretRight} from '@phosphor-icons/react';
import {memo, useCallback} from 'react';
import {TagPill} from '../../../../components/Tags';
import {InlineNameEdit} from '../../components/InlineNameEdit';
import {VcsEmblem} from '../../components/VcsEmblem';
import {File as FileComponent} from '../../File';
import {useDraggableFile} from '../../hooks/useDraggableFile';
import {useShowFileContextMenu} from '../../FileContextMenuProvider';
import {useMiddleClickOpensTab, useOpenFileAction} from '../../OpenFileProvider';
import {useTranslation} from '../../../../i18n';
import {ROW_HEIGHT, TABLE_PADDING_X} from './useTable';

/** Indentation per level of an expanded folder, in pixels. */
const EXPAND_INDENT = 16;

interface Rename {
	onSave: (newName: string) => Promise<void>;
	onCancel: () => void;
}

interface TableRowProps {
	columnSizingKey: string;
	row: Row<File>;
	file: File;
	files: File[];
	/** Position in the flat file order, for row striping. */
	index: number;
	/** Position among the virtualized items, which also count section headers. */
	virtualIndex: number;
	isSelected: boolean;
	isFocused: boolean;
	isPreviousSelected: boolean;
	isNextSelected: boolean;
	measureRef: (node: HTMLElement | null) => void;
	/** The selection to drag with this row; empty unless the row is selected. */
	selectedFiles: File[];
	/** Set only on the row being renamed. */
	rename?: Rename;
	/** Git state shown as an emblem; `null` outside a working tree or when unchanged. */
	vcsStatus: VcsFileStatus | null;
	/** Opens or closes a folder row in place. */
	onToggleExpanded: (file: File) => void;
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
	virtualIndex,
	isSelected,
	isFocused: _isFocused,
	isPreviousSelected,
	isNextSelected,
	measureRef,
	selectedFiles,
	rename,
	vcsStatus,
	onToggleExpanded,
	selectFile
}: TableRowProps) {

	const showContextMenu = useShowFileContextMenu();

	const openFile = useOpenFileAction();

	const handleClick = useCallback(
		(e: React.MouseEvent) => {
			const multi = e.metaKey || e.ctrlKey;
			const range = e.shiftKey;
			selectFile(file, files, multi, range);
		},
		[file, files, selectFile]
	);

	const handleDoubleClick = () => openFile(file);
	const handleAuxClick = useMiddleClickOpensTab(file);

	const handleContextMenu = useCallback(
		async (e: React.MouseEvent) => {
			e.preventDefault();
			e.stopPropagation();

			if (!isSelected) {
				selectFile(file, files, false, false);
			}

			showContextMenu(e, file, isSelected);
		},
		[file, files, isSelected, selectFile, showContextMenu]
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
		selectedFiles: selectedFiles.length > 0 ? selectedFiles : undefined
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
			data-index={virtualIndex}
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
			onAuxClick={handleAuxClick}
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
								<NameCell
									file={file}
									rename={rename}
									vcsStatus={vcsStatus}
									depth={row.depth}
									canExpand={row.getCanExpand()}
									isExpanded={row.getIsExpanded()}
									onToggleExpanded={onToggleExpanded}
								/>
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
const NameCell = memo(function NameCell({
	file,
	rename,
	vcsStatus,
	depth,
	canExpand,
	isExpanded,
	onToggleExpanded
}: {
	file: File;
	rename?: Rename;
	vcsStatus: VcsFileStatus | null;
	depth: number;
	canExpand: boolean;
	isExpanded: boolean;
	onToggleExpanded: (file: File) => void;
}) {
	const {t} = useTranslation('explorer');

	return (
		<div
			className="flex min-w-0 flex-1 items-center gap-2"
			style={{paddingLeft: depth * EXPAND_INDENT}}
		>
			{/* Expand toggle; files keep the space so names line up */}
			<button
				type="button"
				tabIndex={-1}
				aria-label={isExpanded ? t('list.collapseFolder') : t('list.expandFolder')}
				aria-expanded={canExpand ? isExpanded : undefined}
				className={clsx(
					'text-ink-faint hover:text-ink hover:bg-app-hover -mr-1 flex size-4 flex-shrink-0 items-center justify-center rounded-sm',
					!canExpand && 'invisible'
				)}
				onClick={(e) => {
					e.stopPropagation();
					onToggleExpanded(file);
				}}
				onDoubleClick={(e) => e.stopPropagation()}
			>
				<CaretRight
					size={10}
					weight="bold"
					className={clsx('transition-transform', isExpanded && 'rotate-90')}
				/>
			</button>

			{/* File icon */}
			<div className="relative flex-shrink-0">
				<FileComponent.Thumb file={file} size={20} />
				{vcsStatus && (
					<VcsEmblem
						status={vcsStatus}
						size="sm"
						className="absolute -bottom-0.5 -right-1"
					/>
				)}
			</div>

			{/* File name or inline edit */}
			{rename ? (
				<InlineNameEdit
					file={file}
					onSave={rename.onSave}
					onCancel={rename.onCancel}
					className="min-w-0 flex-1"
				/>
			) : (
				<span className="text-ink truncate text-sm">
					{file.name}
					{file.extension && `.${file.extension}`}
				</span>
			)}

			{/* Tags (inline, compact) - hide when renaming */}
			{!rename && file.tags && file.tags.length > 0 && (
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
