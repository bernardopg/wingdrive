import {useDroppable} from '@dnd-kit/core';
import type {File} from '@wingdrive/ts-client';
import {isVirtualFile} from '@wingdrive/ts-client';
import clsx from 'clsx';
import {memo} from 'react';
import {TagDot} from '../../../../components/Tags';
import {InlineNameEdit} from '../../components/InlineNameEdit';
import {VolumeSizeBar} from '../../components/VolumeSizeBar';
import {useExplorer} from '../../context';
import {File as FileComponent} from '../../File';
import {useDraggableFile} from '../../hooks/useDraggableFile';
import {useFileContextMenu} from '../../hooks/useFileContextMenu';
import {useOpenFile} from '../../hooks/useOpenFile';
import {useSelection} from '../../SelectionContext';
import {formatBytes} from '../../utils';

interface FileCardProps {
	file: File;
	fileIndex: number;
	allFiles: File[];
	selected: boolean;
	focused: boolean;
	selectedFiles: File[];
	selectFile: (
		file: File,
		files: File[],
		multi?: boolean,
		range?: boolean
	) => void;
}

export const FileCard = memo(function FileCard({
	file,
	fileIndex,
	allFiles,
	selected,
	focused: _focused,
	selectedFiles,
	selectFile
}: FileCardProps) {
	const {viewSettings} = useExplorer();
	const {gridSize, showFileSize} = viewSettings;
	const {renamingFileId, saveRename, cancelRename} = useSelection();

	const isRenaming = renamingFileId === file.id;

	const contextMenu = useFileContextMenu({
		file,
		selectedFiles,
		selected
	});

	const openFile = useOpenFile();

	const handleClick = (e: React.MouseEvent) => {
		const multi = e.metaKey || e.ctrlKey;
		const range = e.shiftKey;
		selectFile(file, allFiles, multi, range);
	};

	const handleDoubleClick = () => openFile(file);

	const handleContextMenu = async (e: React.MouseEvent) => {
		e.preventDefault();
		e.stopPropagation();

		if (!selected) {
			selectFile(file, allFiles, false, false);
		}

		await contextMenu.show(e);
	};

	const {
		attributes,
		listeners,
		setNodeRef: setDragNodeRef,
		isDragging: dndIsDragging
	} = useDraggableFile({
		file,
		selectedFiles:
			selected && selectedFiles.length > 0 ? selectedFiles : undefined,
		gridSize
	});

	// Make folders droppable
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

	// Combine refs for folders that are both draggable and droppable
	const setNodeRef = (node: HTMLElement | null) => {
		setDragNodeRef(node);
		if (isFolder) setDropNodeRef(node);
	};

	const thumbSize = Math.max(gridSize * 0.6, 60);

	// Check if this is a virtual volume file
	const isVolume =
		isVirtualFile(file) &&
		(file as any)._virtual?.type === 'volume' &&
		(file as any)._virtual?.data;

	// Extract volume data
	const volumeData = isVolume ? (file as any)._virtual.data : null;
	const hasVolumeCapacity =
		volumeData?.total_capacity != null &&
		volumeData?.available_space != null &&
		volumeData.total_capacity > 0;

	return (
		<div
			ref={setNodeRef}
			{...listeners}
			{...attributes}
			data-file-id={file.id}
			data-index={fileIndex}
			data-selectable="true"
			aria-selected={selected}
			tabIndex={-1}
			className="relative outline-none focus:outline-none"
		>
			{/* Drop indicator for folders */}
			{isFolder && isDropOver && (
				<div className="ring-accent pointer-events-none absolute inset-0 z-10 rounded-lg ring-2 ring-inset" />
			)}
			<FileComponent
				file={file}
				selected={selected && !dndIsDragging}
				onClick={handleClick}
				onDoubleClick={handleDoubleClick}
				onContextMenu={handleContextMenu}
				layout="column"
				className={clsx(
					'flex flex-col items-center gap-2 rounded-lg p-1 transition-all',
					dndIsDragging && 'opacity-40',
					isFolder && isDropOver && 'bg-accent/10'
				)}
			>
				<div
					className={clsx(
						'rounded-lg p-2',
						selected && !dndIsDragging
							? 'bg-app-box'
							: 'bg-transparent'
					)}
				>
					<FileComponent.Thumb file={file} size={thumbSize} />
				</div>
				<div className="flex w-full flex-col items-center">
					{isRenaming ? (
						<InlineNameEdit
							file={file}
							onSave={saveRename}
							onCancel={cancelRename}
							className="max-w-full"
						/>
					) : (
						<div
							className={clsx(
								'inline-block max-w-full truncate rounded-md px-2 py-0.5 text-sm',
								selected && !dndIsDragging
									? 'bg-accent text-white'
									: 'text-ink'
							)}
						>
							{file.name}
							{file.extension && `.${file.extension}`}
						</div>
					)}

					{/* Volume size bar */}
					{showFileSize && hasVolumeCapacity && (
						<VolumeSizeBar
							totalBytes={Number(volumeData.total_capacity)}
							availableBytes={Number(volumeData.available_space)}
							className="mt-1.5"
						/>
					)}

					{/* Regular file size */}
					{showFileSize && !hasVolumeCapacity && file.size > 0 && (
						<div className="text-ink-dull mt-0.5 text-xs">
							{formatBytes(file.size)}
						</div>
					)}

					{/* Tag Indicators */}
					{file.tags && file.tags.length > 0 && (
						<div
							className="mt-1 flex items-center gap-1"
							title={file.tags
								.map((t) => t.canonical_name)
								.join(', ')}
						>
							{file.tags.slice(0, 3).map((tag) => (
								<TagDot
									key={tag.id}
									color={tag.color || '#3B82F6'}
									tooltip={tag.canonical_name}
								/>
							))}
							{file.tags.length > 3 && (
								<span className="text-ink-faint text-[10px] font-medium">
									+{file.tags.length - 3}
								</span>
							)}
						</div>
					)}
				</div>
			</FileComponent>
		</div>
	);
});
