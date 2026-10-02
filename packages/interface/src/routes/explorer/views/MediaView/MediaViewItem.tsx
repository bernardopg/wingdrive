import type {File} from '@wingdrive/ts-client';
import clsx from 'clsx';
import {memo} from 'react';
import {File as FileComponent} from '../../File';
import {useFileContextMenu} from '../../hooks/useFileContextMenu';
import {useOpenFile} from '../../hooks/useOpenFile';
import {useSelection} from '../../SelectionContext';

function formatDuration(seconds: number): string {
	const mins = Math.floor(seconds / 60);
	const secs = Math.floor(seconds % 60);
	return `${mins}:${String(secs).padStart(2, '0')}`;
}

interface MediaViewItemProps {
	file: File;
	allFiles: File[];
	selected: boolean;
	focused: boolean;
	onSelect: (
		file: File,
		files: File[],
		multi?: boolean,
		range?: boolean
	) => void;
	size: number;
}

export const MediaViewItem = memo(function MediaViewItem({
	file,
	allFiles,
	selected,
	focused,
	onSelect,
	size
}: MediaViewItemProps) {
	const {selectedFiles} = useSelection();

	const contextMenu = useFileContextMenu({
		file,
		selectedFiles,
		selected
	});

	const handleClick = (e: React.MouseEvent) => {
		const multi = e.metaKey || e.ctrlKey;
		const range = e.shiftKey;
		onSelect(file, allFiles, multi, range);
	};

	const handleContextMenu = async (e: React.MouseEvent) => {
		e.preventDefault();
		e.stopPropagation();

		if (!selected) {
			onSelect(file, allFiles, false, false);
		}

		await contextMenu.show(e);
	};

	const openFile = useOpenFile();
	const handleDoubleClick = () => openFile(file);

	return (
		<div
			data-file-id={file.id}
			tabIndex={-1}
			className={clsx(
				'group relative h-full w-full cursor-pointer overflow-hidden transition-all outline-none focus:outline-none',
				selected && 'ring-accent ring-2 ring-inset',
				focused && !selected && 'ring-accent/50 ring-2 ring-inset'
			)}
			onClick={handleClick}
			onDoubleClick={handleDoubleClick}
			onContextMenu={handleContextMenu}
		>
			<FileComponent.Thumb
				file={file}
				size={size}
				className="h-full w-full"
				frameClassName="w-full h-full object-cover"
				iconScale={0.5}
				squareMode={true}
			/>

			{/* Selection overlay */}
			{selected && (
				<div className="bg-accent/10 pointer-events-none absolute inset-0" />
			)}

			{/* Video duration badge */}
			{file.video_media_data?.duration_seconds && (
				<div className="absolute right-1 bottom-1 rounded bg-black/80 px-1.5 py-0.5 text-[10px] font-medium text-white tabular-nums backdrop-blur-sm">
					{formatDuration(file.video_media_data.duration_seconds)}
				</div>
			)}

			{/* Hover overlay with file name */}
			<div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/70 to-transparent px-2 py-1.5 opacity-0 transition-opacity group-hover:opacity-100">
				<div className="truncate text-xs font-medium text-white">
					{file.name}
					{file.extension && `.${file.extension}`}
				</div>
			</div>

			{/* Selection checkbox (top-left corner, always visible when selected) */}
			{selected && (
				<div className="bg-accent absolute top-1 left-1 flex h-5 w-5 items-center justify-center rounded-full">
					<svg
						className="h-3 w-3 text-white"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							strokeLinecap="round"
							strokeLinejoin="round"
							strokeWidth={3}
							d="M5 13l4 4L19 7"
						/>
					</svg>
				</div>
			)}
		</div>
	);
});
