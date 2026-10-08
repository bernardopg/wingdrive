import {
	ArrowSquareOut,
	Copy,
	Crop,
	Eye,
	FileText,
	FileVideo,
	FilmStrip,
	FolderOpen,
	FolderPlus,
	Image,
	MagnifyingGlass,
	Microphone,
	Pencil,
	Scissors,
	ShareNetwork,
	FileArchive,
	Package,
	Link,
	FilePlus,
	TerminalWindow,
	Sparkle,
	Stack,
	Tag as TagIconComponent,
	TextAa,
	Trash,
	Video,
	Waveform
} from '@phosphor-icons/react';
import {toast} from '@wingdrive/primitives';
import type {File} from '@wingdrive/ts-client';
import {getContentKind, isVirtualFile} from '@wingdrive/ts-client';
import {usePlatform} from '../../../contexts/PlatformContext';
import {useLibraryMutation} from '../../../contexts/WingDriveContext';
import {useClipboard} from '../../../hooks/useClipboard';
import {useContextMenu} from '../../../hooks/useContextMenu';
import {useOpenWith} from '../../../hooks/useOpenWith';
import {useRefetchTagQueries} from '../../../hooks/useRefetchTagQueries';
import {useExplorer} from '../context';
import {selectionCapabilities} from '../fileCapabilities';
import {useSelection} from '../SelectionContext';
import {COMPRESS_FORMATS, isArchiveName} from './archiveName';
import {useArchiveActions} from './useArchiveActions';
import {useCreateEntry} from './useCreateEntry';
import {useCreateFolder} from './useCreateFolder';
import {useDeleteFiles} from './useDeleteFiles';
import {useDuplicateFiles} from './useDuplicateFiles';
import {useOpenFiles} from './useOpenFile';
import {useOpenTerminal} from './useOpenTerminal';
import {usePasteFiles} from './usePasteFiles';
import {revealLabel} from '../../../util/keybinds/platform';

interface UseFileContextMenuProps {
	file?: File | null;
	selectedFiles: File[];
	selected: boolean;
}

export function useFileContextMenu({
	file,
	selectedFiles,
	selected
}: UseFileContextMenuProps) {
	const {operationalPath, mode, openQuickPreview} = useExplorer();
	const platform = usePlatform();
	const refetchTagQueries = useRefetchTagQueries();

	const {deleteFiles} = useDeleteFiles();
	const {duplicateFiles} = useDuplicateFiles();
	const unapplyTags = useLibraryMutation('tags.unapply', {
		onSuccess: refetchTagQueries
	});
	const createFolder = useLibraryMutation('files.createFolder');
	const createEmptyFolder = useCreateFolder();
	const regenerateThumbnail = useLibraryMutation(
		'media.thumbnail.regenerate'
	);
	const extractText = useLibraryMutation('media.ocr.extract');
	const transcribeAudio = useLibraryMutation('media.speech.transcribe');
	const generateThumbstrip = useLibraryMutation('media.thumbstrip.generate');
	const generateProxy = useLibraryMutation('media.proxy.generate');

	// Helper to run a mutation on each target file
	const forEachTarget = async (
		targets: File[],
		fn: (f: File) => Promise<unknown>
	) => {
		for (const f of targets) {
			try {
				await fn(f);
			} catch (err) {
				console.error(`Failed for ${f.name}:`, err);
			}
		}
	};
	const clipboard = useClipboard();
	const pasteFiles = usePasteFiles();
	const openFiles = useOpenFiles();
	const {startRename} = useSelection();

	// Get physical paths for file opening
	const getPhysicalPaths = () => {
		const targets =
			selected && selectedFiles.length > 0 ? selectedFiles : [file];
		return targets
			.filter(
				(f): f is File =>
					f != null &&
					f.wing_path != null &&
					'Physical' in f.wing_path
			)
			.map((f) => (f.wing_path as any).Physical.path);
	};

	const physicalPaths = getPhysicalPaths();
	const {apps, openWithApp, openMultipleWithApp, canSetDefault, setDefaultApp} =
		useOpenWith(physicalPaths);
	const openTerminal = useOpenTerminal();
	const {newFile, newLink} = useCreateEntry();
	const archive = useArchiveActions();

	// A right-click on an unselected item acts on that item alone.
	const targetFiles = selected && selectedFiles.length > 0
		? selectedFiles
		: file
			? [file]
			: [];
	const capabilities = selectionCapabilities(targetFiles);
	const hasVirtualFiles = capabilities.hasVirtual;
	const getTargetFiles = () => capabilities.operable;

	return useContextMenu({
		items: [
			{
				icon: Eye,
				label: 'Quick Look',
				onClick: () => {
					if (!file) return;
					openQuickPreview(file.id);
				},
				keybind: 'Space',
				condition: () => !!file
			},
			{
				icon: FolderOpen,
				label: targetFiles.length > 1 ? `Open ${targetFiles.length} Items` : 'Open',
				onClick: () => void openFiles(targetFiles),
				keybindId: 'explorer.openFile',
				condition: () =>
					!!file &&
					(isVirtualFile(file) ||
						file.kind === 'Directory' ||
						file.kind === 'File' ||
						file.kind === 'Symlink')
			},
			{
				type: 'submenu',
				icon: ArrowSquareOut,
				label: 'Open With',
				condition: () =>
					!!file &&
					file.kind === 'File' &&
					'Physical' in file.wing_path &&
					apps.length > 0,
				submenu: apps.map((app) => ({
					label: app.is_default ? `${app.name} (default)` : app.name,
					iconUrl: app.icon,
					onClick: async () => {
						if (!file) return;
						if (selected && selectedFiles.length > 1) {
							await openMultipleWithApp(physicalPaths, app.id);
						} else if ('Physical' in file.wing_path) {
							await openWithApp(file.wing_path.Physical.path, app.id);
						}
					}
				}))
			},
			{
				type: 'submenu',
				icon: ArrowSquareOut,
				label: 'Always Open With',
				condition: () =>
					canSetDefault &&
					!!file &&
					file.kind === 'File' &&
					physicalPaths.length === 1 &&
					apps.some((app) => !app.is_default),
				submenu: apps.filter((app) => !app.is_default).map((app) => ({
					label: app.name,
					iconUrl: app.icon,
					onClick: () => setDefaultApp(physicalPaths[0], app.id, app.name)
				}))
			},
			{
				icon: MagnifyingGlass,
				label: revealLabel(),
				onClick: async () => {
					if (!file) return;
					// Extract the physical path from WingPath
					if ('Physical' in file.wing_path) {
						const physicalPath = file.wing_path.Physical.path;
						if (platform.revealFile) {
							try {
								await platform.revealFile(physicalPath);
							} catch (err) {
								console.error('Failed to reveal file:', err);
								toast.error(`Failed to reveal file: ${err}`);
							}
						} else {
							console.log(
								'revealFile not supported on this platform'
							);
						}
					} else {
						console.log('Cannot reveal non-physical file');
					}
				},
				keybindId: 'explorer.revealInNativeExplorer',
				condition: () =>
					!!file &&
					'Physical' in file.wing_path &&
					!!platform.revealFile
			},
			{
				icon: TerminalWindow,
				label: 'Open Terminal Here',
				onClick: () => {
					void openTerminal?.(file?.wing_path);
				},
				condition: () =>
					!!openTerminal &&
					!!file &&
					file.kind === 'Directory' &&
					'Physical' in file.wing_path &&
					targetFiles.length === 1
			},
			{
				icon: ShareNetwork,
				label:
					selected && selectedFiles.length > 1
						? `Share ${selectedFiles.length} items`
						: 'Share',
				onClick: async () => {
					const paths = physicalPaths;
					if (paths.length === 0) {
						console.warn('No physical files to share');
						return;
					}
					if (platform.shareFiles) {
						try {
							await platform.shareFiles(paths);
						} catch (err) {
							console.error('Failed to share files:', err);
							toast.error(`Failed to share: ${err}`);
						}
					}
				},
				condition: () =>
					physicalPaths.length > 0 && !!platform.shareFiles
			},
			{type: 'separator'},
			{
				icon: Pencil,
				label: 'Rename',
				onClick: () => {
					if (!file) return;
					startRename(file.id, file);
				},
				keybindId: 'explorer.renameFile',
				condition: () =>
					!!file && targetFiles.length === 1 && capabilities.canRename
			},
			{
				icon: FolderPlus,
				label: 'New Folder',
				onClick: () => {
					void createEmptyFolder();
				},
				condition: () => !!operationalPath
			},
			{
				icon: FilePlus,
				label: 'New File',
				onClick: () => {
					void newFile();
				},
				condition: () => !!operationalPath && 'Physical' in operationalPath
			},
			{
				icon: Link,
				label: 'Create Link',
				onClick: () => {
					if (file) void newLink(file);
				},
				condition: () =>
					!!file &&
					targetFiles.length === 1 &&
					'Physical' in file.wing_path &&
					!!operationalPath &&
					'Physical' in operationalPath
			},
			{
				icon: FileArchive,
				label: 'Extract Here',
				onClick: () => {
					if (file) archive.extract(file);
				},
				condition: () =>
					!!file &&
					targetFiles.length === 1 &&
					file.kind === 'File' &&
					'Physical' in file.wing_path &&
					isArchiveName(file.wing_path.Physical.path)
			},
			{
				icon: FileArchive,
				label: 'Extract To...',
				onClick: () => {
					if (file) void archive.extractTo(file);
				},
				condition: () =>
					archive.canPickDirectory &&
					!!file &&
					targetFiles.length === 1 &&
					file.kind === 'File' &&
					'Physical' in file.wing_path &&
					isArchiveName(file.wing_path.Physical.path)
			},
			{
				type: 'submenu',
				icon: Package,
				label: 'Compress',
				condition: () =>
					targetFiles.length > 0 &&
					targetFiles.every((f) => 'Physical' in f.wing_path && f.is_local),
				submenu: COMPRESS_FORMATS.map(({format, label}) => ({
					label,
					onClick: () => archive.compress(targetFiles, format)
				}))
			},
			{
				icon: FolderPlus,
				label: 'New Folder with Items',
				onClick: async () => {
					if (!operationalPath) return;
					const targets = getTargetFiles();
					if (targets.length === 0) return;

					try {
						const result = await createFolder.mutateAsync({
							parent: operationalPath,
							name: 'New Folder',
							items: targets.map((f) => f.wing_path)
						});
						console.log('Created folder with items:', result);
					} catch (err) {
						console.error(
							'Failed to create folder with items:',
							err
						);
						toast.error(`Failed to create folder: ${err}`);
					}
				},
				condition: () => !!operationalPath && capabilities.canCopy
			},
			{type: 'separator'},
			{
				icon: Copy,
				label:
					selected && selectedFiles.length > 1
						? `Copy ${selectedFiles.length} items`
						: 'Copy',
				onClick: () => {
					const targets = getTargetFiles();
					if (targets.length === 0) {
						console.warn('Cannot copy virtual files');
						return;
					}
					const sdPaths = targets.map((f) => f.wing_path);
					clipboard.copyFiles(
						sdPaths,
						operationalPath,
						targets.every((file) => file.is_local)
					);
				},
				keybindId: 'explorer.copy',
				condition: () => capabilities.canCopy
			},
			{
				icon: Scissors,
				label:
					selected && selectedFiles.length > 1
						? `Cut ${selectedFiles.length} items`
						: 'Cut',
				onClick: () => {
					const targets = getTargetFiles();
					if (targets.length === 0) {
						console.warn('Cannot cut virtual files');
						return;
					}
					const sdPaths = targets.map((f) => f.wing_path);
					clipboard.cutFiles(
						sdPaths,
						operationalPath,
						targets.every((file) => file.is_local)
					);
				},
				keybindId: 'explorer.cut',
				condition: () => capabilities.canCopy
			},
			{
				icon: Copy,
				label: 'Paste',
				onClick: () => {
					void pasteFiles(operationalPath);
				},
				keybindId: 'explorer.paste',
				condition: () => clipboard.canPaste() && !!operationalPath
			},
			{
				icon: Copy,
				label:
					selected && selectedFiles.length > 1
						? `Duplicate ${selectedFiles.length} items`
						: 'Duplicate',
				onClick: async () => {
					const targets = getTargetFiles();
					if (targets.length === 0) return;
					await duplicateFiles(targets);
				},
				keybindId: 'explorer.duplicate',
				condition: () => capabilities.canDuplicate
			},
			// Media Processing submenu
			{
				type: 'submenu',
				icon: Image,
				label: 'Image Processing',
				condition: () => !!file && getContentKind(file) === 'image',
				submenu: [
					{
						icon: Sparkle,
						label: 'Generate Blurhash',
						onClick: async () => {
							const targets = getTargetFiles();
							await forEachTarget(targets, (f) =>
								regenerateThumbnail.mutateAsync({
									entry_uuid: f.id,
									variants: null,
									force: false
								})
							);
						},
						condition: () =>
							!!file && !file.image_media_data?.blurhash
					},
					{
						icon: Crop,
						label: 'Regenerate Thumbnail',
						onClick: async () => {
							const targets = getTargetFiles();
							await forEachTarget(targets, (f) =>
								regenerateThumbnail.mutateAsync({
									entry_uuid: f.id,
									variants: null,
									force: true
								})
							);
						}
					},
					{
						icon: TextAa,
						label: 'Extract Text (OCR)',
						onClick: async () => {
							const targets = getTargetFiles();
							await forEachTarget(targets, (f) =>
								extractText.mutateAsync({
									entry_uuid: f.id,
									languages: null,
									force: false
								})
							);
						}
					}
				]
			},
			{
				type: 'submenu',
				icon: Video,
				label: 'Video Processing',
				condition: () => !!file && getContentKind(file) === 'video',
				submenu: [
					{
						icon: FilmStrip,
						label: 'Generate Thumbstrip',
						onClick: async () => {
							const targets = getTargetFiles();
							await forEachTarget(targets, (f) =>
								generateThumbstrip.mutateAsync({
									entry_uuid: f.id,
									variants: null,
									force: false
								})
							);
						},
						condition: () =>
							!!file &&
							!file.sidecars?.some((s) => s.kind === 'thumbstrip')
					},
					{
						icon: Sparkle,
						label: 'Generate Blurhash',
						onClick: async () => {
							const targets = getTargetFiles();
							await forEachTarget(targets, (f) =>
								regenerateThumbnail.mutateAsync({
									entry_uuid: f.id,
									variants: null,
									force: false
								})
							);
						},
						condition: () =>
							!!file && !file.video_media_data?.blurhash
					},
					{
						icon: Crop,
						label: 'Regenerate Thumbnail',
						onClick: async () => {
							const targets = getTargetFiles();
							await forEachTarget(targets, (f) =>
								regenerateThumbnail.mutateAsync({
									entry_uuid: f.id,
									variants: null,
									force: true
								})
							);
						}
					},
					{
						icon: Waveform,
						label: 'Extract Subtitles',
						onClick: async () => {
							const targets = getTargetFiles();
							await forEachTarget(targets, (f) =>
								transcribeAudio.mutateAsync({
									entry_uuid: f.id,
									model: null,
									language: null
								})
							);
						}
					},
					{
						icon: FileVideo,
						label: 'Generate Proxy',
						onClick: async () => {
							const targets = getTargetFiles();
							await forEachTarget(targets, (f) =>
								generateProxy.mutateAsync({
									entry_uuid: f.id,
									resolution: null,
									use_hardware_accel: null,
									force: false
								})
							);
						}
					}
				]
			},
			{
				type: 'submenu',
				icon: Microphone,
				label: 'Audio Processing',
				condition: () => !!file && getContentKind(file) === 'audio',
				submenu: [
					{
						icon: TextAa,
						label: 'Transcribe Audio',
						onClick: async () => {
							const targets = getTargetFiles();
							await forEachTarget(targets, (f) =>
								transcribeAudio.mutateAsync({
									entry_uuid: f.id,
									model: 'whisper-base',
									language: null
								})
							);
						}
					}
				]
			},
			{
				type: 'submenu',
				icon: FileText,
				label: 'Document Processing',
				condition: () =>
					!!file &&
					file.kind === 'File' &&
					['pdf', 'doc', 'docx'].includes(file.extension || ''),
				submenu: [
					{
						icon: TextAa,
						label: 'Extract Text (OCR)',
						onClick: async () => {
							const targets = getTargetFiles();
							await forEachTarget(targets, (f) =>
								extractText.mutateAsync({
									entry_uuid: f.id,
									languages: null,
									force: false
								})
							);
						}
					},
					{
						icon: Crop,
						label: 'Regenerate Thumbnail',
						onClick: async () => {
							const targets = getTargetFiles();
							await forEachTarget(targets, (f) =>
								regenerateThumbnail.mutateAsync({
									entry_uuid: f.id,
									variants: null,
									force: true
								})
							);
						}
					}
				]
			},
			// Batch operations submenu
			{
				type: 'submenu',
				icon: Stack,
				label: `Process ${selectedFiles.length} Items`,
				condition: () =>
					selected && selectedFiles.length > 1 && !hasVirtualFiles,
				submenu: [
					{
						icon: Crop,
						label: 'Regenerate All Thumbnails',
						onClick: async () => {
							await forEachTarget(getTargetFiles(), (f) =>
								regenerateThumbnail.mutateAsync({
									entry_uuid: f.id,
									variants: null,
									force: true
								})
							);
						}
					},
					{
						icon: Sparkle,
						label: 'Generate Blurhashes',
						onClick: async () => {
							await forEachTarget(getTargetFiles(), (f) =>
								regenerateThumbnail.mutateAsync({
									entry_uuid: f.id,
									variants: null,
									force: false
								})
							);
						},
						keybind: '⌘⇧B'
					},
					{
						icon: TextAa,
						label: 'Extract Text (OCR)',
						onClick: async () => {
							await forEachTarget(getTargetFiles(), (f) =>
								extractText.mutateAsync({
									entry_uuid: f.id,
									languages: null,
									force: false
								})
							);
						}
					}
				]
			},
			{
				icon: TagIconComponent,
				label:
					selected && selectedFiles.length > 1
						? `Remove tag from ${selectedFiles.length} items`
						: 'Remove tag',
				onClick: async () => {
					if (mode.type !== 'tag') return;
					const targets = getTargetFiles();
					if (targets.length === 0) return;
					try {
						await unapplyTags.mutateAsync({
							entry_ids: targets.map((f) => f.id),
							tag_ids: [mode.tagId]
						});
					} catch (err) {
						console.error('Failed to remove tag:', err);
						toast.error(`Failed to remove tag: ${err}`);
					}
				},
				condition: () => mode.type === 'tag' && !hasVirtualFiles
			},
			{type: 'separator'},
			{
				icon: Trash,
				label:
					selected && selectedFiles.length > 1
						? `Delete ${selectedFiles.length} items`
						: 'Delete',
				onClick: async () => {
					const targets = getTargetFiles();
					await deleteFiles(targets, false);
				},
				keybindId: 'explorer.delete',
				variant: 'danger' as const,
				condition: () => capabilities.canDelete
			}
		]
	});
}
