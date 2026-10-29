import {
	AppWindow,
	ArrowBendUpRight,
	ArrowSquareOut,
	Copy,
	CopySimple,
	Crop,
	Eye,
	FileText,
	FileVideo,
	FilmStrip,
	FolderOpen,
	FolderPlus,
	GitDiff,
	Image,
	Lightning,
	MagnifyingGlass,
	Microphone,
	Pencil,
	Scissors,
	ShareNetwork,
	FileArchive,
	Package,
	PaperPlaneTilt,
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
import {useContextMenu, type ContextMenuItem} from '../../../hooks/useContextMenu';
import {useOpenWith} from '../../../hooks/useOpenWith';
import {useRefetchTagQueries} from '../../../hooks/useRefetchTagQueries';
import {useBatchRenameDialog} from '../../../components/modals/BatchRenameModal';
import {useCompareFoldersDialog} from '../../../components/modals/CompareFoldersModal';
import {useDuplicatesDialog} from '../../../components/modals/DuplicatesModal';
import {useExplorer} from '../context';
import {selectionCapabilities} from '../fileCapabilities';
import {useSelection} from '../SelectionContext';
import {COMPRESS_FORMATS, isArchiveName} from './archiveName';
import {useArchiveActions} from './useArchiveActions';
import {contextActionMenus, useContextActions} from './useContextActions';
import {useCopyMoveTo} from './useCopyMoveTo';
import {useWingdrop} from './useWingdrop';
import {useCreateEntry} from './useCreateEntry';
import {useCreateFolder} from './useCreateFolder';
import {useDeleteFiles} from './useDeleteFiles';
import {useDuplicateFiles} from './useDuplicateFiles';
import {useOpenFiles} from './useOpenFile';
import {useOpenInNewWindow} from './useOpenInNewWindow';
import {useOpenTerminal} from './useOpenTerminal';
import {usePasteFiles} from './usePasteFiles';
import {revealLabel} from '../../../util/keybinds/platform';
import {useTranslation} from '../../../i18n';

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
	const {t} = useTranslation('explorer');
	const {operationalPath, mode, openQuickPreview, currentFiles} = useExplorer();
	const platform = usePlatform();
	const refetchTagQueries = useRefetchTagQueries();

	const {deleteFiles} = useDeleteFiles();
	const openBatchRename = useBatchRenameDialog();
	const openDuplicates = useDuplicatesDialog();
	const openCompareFolders = useCompareFoldersDialog();
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

	const openTargets = (
		selected && selectedFiles.length > 0 ? selectedFiles : [file]
	).filter(
		(f): f is File =>
			f != null && f.wing_path != null && 'Physical' in f.wing_path
	);

	// Paths on this machine, for the platform calls that take one. Another
	// device's path would name a different file here, or none.
	const physicalPaths = openTargets.flatMap((f) =>
		f.is_local && 'Physical' in f.wing_path ? [f.wing_path.Physical.path] : []
	);
	const allTargetsLocal =
		physicalPaths.length > 0 && physicalPaths.length === openTargets.length;
	const {apps, isLoading: appsLoading, openWithApp, openMultipleWithApp, canSetDefault, setDefaultApp} =
		useOpenWith(physicalPaths);
	const contextActions = useContextActions(allTargetsLocal ? physicalPaths : []);
	const openTerminal = useOpenTerminal();
	const openInNewWindow = useOpenInNewWindow();
	const {newFile, newLink} = useCreateEntry();
	const archive = useArchiveActions();
	const copyMoveTo = useCopyMoveTo();
	const wingdrop = useWingdrop();

	// A right-click on an unselected item acts on that item alone.
	const targetFiles = selected && selectedFiles.length > 0
		? selectedFiles
		: file
			? [file]
			: [];
	const capabilities = selectionCapabilities(targetFiles);
	const hasVirtualFiles = capabilities.hasVirtual;
	const getTargetFiles = () => capabilities.operable;

	const localFolders = targetFiles.flatMap((f) =>
		f.kind === 'Directory' && 'Physical' in f.wing_path
			? [f.wing_path.Physical.path]
			: []
	);

	// The empty-space menu covers the folder being browsed.
	const duplicateScanRoot =
		targetFiles.length === 1 && localFolders.length === 1
			? localFolders[0]
			: null;

	// Inside an archive only opening and extracting apply; the rest would
	// try to change files that are not on disk.
	const inArchive =
		targetFiles.length > 0
			? targetFiles.every((f) => !!f.archive)
			: currentFiles.some((f) => !!f.archive);
	const archiveItems: ContextMenuItem[] = [];
	const forArchive = (item: ContextMenuItem) => {
		archiveItems.push(item);
		return item;
	};

	const items: ContextMenuItem[] = [
		forArchive({
			icon: Eye,
			label: t('contextMenu.quickLook'),
			onClick: () => {
				if (!file) return;
				openQuickPreview(file.id);
			},
			keybind: 'Space',
			condition: () => !!file
		}),
		forArchive({
			icon: FolderOpen,
			label: targetFiles.length > 1 ? t('contextMenu.openItems', {count: targetFiles.length}) : t('contextMenu.open'),
			onClick: () => void openFiles(targetFiles),
			keybindId: 'explorer.openFile',
			condition: () =>
				!!file &&
				(isVirtualFile(file) ||
					file.kind === 'Directory' ||
					file.kind === 'File' ||
					file.kind === 'Symlink')
		}),
		{
			icon: AppWindow,
			label: t('contextMenu.openInNewWindow'),
			onClick: () => {
				void openInNewWindow?.(file?.wing_path);
			},
			keybindId: 'explorer.openInNewWindow',
			condition: () =>
				!!openInNewWindow &&
				!!file &&
				file.kind === 'Directory' &&
				'Physical' in file.wing_path &&
				targetFiles.length === 1
		},
		{
			type: 'submenu',
			icon: ArrowSquareOut,
			label: t('contextMenu.openWith'),
			condition: () =>
				!!file &&
				file.kind === 'File' &&
				file.is_local &&
				'Physical' in file.wing_path &&
				allTargetsLocal &&
				apps.length > 0,
			submenu: apps.map((app) => ({
				label: app.is_default ? t('contextMenu.defaultApp', {name: app.name}) : app.name,
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
			label: t('contextMenu.alwaysOpenWith'),
			condition: () =>
				canSetDefault &&
				!!file &&
				file.kind === 'File' &&
				allTargetsLocal &&
				physicalPaths.length === 1 &&
				apps.some((app) => !app.is_default),
			submenu: apps.filter((app) => !app.is_default).map((app) => ({
				label: app.name,
				iconUrl: app.icon,
				onClick: () => setDefaultApp(physicalPaths[0], app.id, app.name)
			}))
		},
		...contextActionMenus(
			contextActions.actions,
			t('contextMenu.actions'),
			Lightning,
			contextActions.run
		),
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
							toast.error(t('errors.failedToRevealFile', {error: err}));
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
				file.is_local &&
				'Physical' in file.wing_path &&
				!!platform.revealFile
		},
		{
			icon: TerminalWindow,
			label: t('contextMenu.openTerminalHere'),
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
					? t('contextMenu.shareItems', {count: selectedFiles.length})
		: t('contextMenu.share'),
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
						toast.error(t('errors.failedToShare', {error: err}));
					}
				}
			},
			condition: () => allTargetsLocal && !!platform.shareFiles
		},
		{type: 'separator'},
		{
			icon: Pencil,
			label: t('contextMenu.rename'),
			onClick: () => {
				if (!file) return;
				startRename(file.id, file);
			},
			keybindId: 'explorer.renameFile',
			condition: () =>
				!!file && targetFiles.length === 1 && capabilities.canRename
		},
		{
			icon: Pencil,
			label: t('contextMenu.renameItems', {count: targetFiles.length}),
			onClick: () => openBatchRename(getTargetFiles()),
			// Renaming needs the same rights as deleting: real, writable entries.
			condition: () => targetFiles.length > 1 && capabilities.canDelete
		},
		{
			icon: GitDiff,
			label: t('contextMenu.compareFolders'),
			onClick: () => openCompareFolders(localFolders[0], localFolders[1]),
			condition: () =>
				targetFiles.length === 2 && localFolders.length === 2
		},
		{
			icon: Copy,
			label: t('contextMenu.findDuplicates'),
			onClick: () => {
				if (duplicateScanRoot) openDuplicates(duplicateScanRoot);
			},
			condition: () => duplicateScanRoot !== null
		},
		{
			icon: FolderPlus,
			label: t('contextMenu.newFolder'),
			onClick: () => {
				void createEmptyFolder();
			},
			condition: () => !!operationalPath
		},
		{
			icon: FilePlus,
			label: t('contextMenu.newFile'),
			onClick: () => {
				void newFile();
			},
			condition: () => !!operationalPath && 'Physical' in operationalPath
		},
		{
			icon: Link,
			label: t('contextMenu.createLink'),
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
		forArchive({
			icon: FileArchive,
			label: t('contextMenu.extractHere'),
			onClick: () => {
				if (file) archive.extract(file);
			},
			condition: () =>
				!!file &&
				targetFiles.length === 1 &&
				'Physical' in file.wing_path &&
				(inArchive || (file.kind === 'File' && isArchiveName(file.wing_path.Physical.path)))
		}),
		forArchive({
			icon: FileArchive,
			label: t('contextMenu.extractTo'),
			onClick: () => {
				if (file) void archive.extractTo(file);
			},
			condition: () =>
				archive.canPickDirectory &&
				!!file &&
				targetFiles.length === 1 &&
				'Physical' in file.wing_path &&
				(inArchive || (file.kind === 'File' && isArchiveName(file.wing_path.Physical.path)))
		}),
		{
			type: 'submenu',
			icon: Package,
			label: t('contextMenu.compress'),
			condition: () =>
				targetFiles.length > 0 &&
				targetFiles.every((f) => 'Physical' in f.wing_path && f.is_local),
			submenu: COMPRESS_FORMATS.map(({format, label}) => ({
				label: t(label),
				onClick: () => archive.compress(targetFiles, format)
			}))
		},
		{
			type: 'submenu',
			icon: PaperPlaneTilt,
			label: t('contextMenu.sendWithWingdrop'),
			condition: () =>
				wingdrop.targets.length > 0 &&
				targetFiles.length > 0 &&
				targetFiles.every((f) => 'Physical' in f.wing_path && f.is_local && !f.archive),
			submenu: wingdrop.targets.map((device) => ({
				label: device.name,
				onClick: () => void wingdrop.sendTo(device, targetFiles)
			}))
		},
		{
			icon: FolderPlus,
			label: t('contextMenu.newFolderWithItems'),
			onClick: async () => {
				if (!operationalPath) return;
				const targets = getTargetFiles();
				if (targets.length === 0) return;

				try {
					const result = await createFolder.mutateAsync({
						parent: operationalPath,
						name: t('contextMenu.newFolder'),
						items: targets.map((f) => f.wing_path)
					});
					console.log('Created folder with items:', result);
				} catch (err) {
					console.error(
						'Failed to create folder with items:',
						err
					);
					toast.error(t('errors.failedToCreateFolder', {error: err}));
				}
			},
			condition: () => !!operationalPath && capabilities.canCopy
		},
		{type: 'separator'},
		{
			icon: Copy,
			label:
				selected && selectedFiles.length > 1
					? t('contextMenu.copyItems', {count: selectedFiles.length})
		: t('contextMenu.copy'),
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
					? t('contextMenu.cutItems', {count: selectedFiles.length})
		: t('contextMenu.cut'),
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
			label: t('contextMenu.paste'),
			onClick: () => {
				void pasteFiles(operationalPath);
			},
			keybindId: 'explorer.paste',
			condition: () => clipboard.canPaste() && !!operationalPath
		},
		{
			icon: CopySimple,
			label: t('contextMenu.copyTo'),
			onClick: () => void copyMoveTo.copyTo(getTargetFiles()),
			condition: () => capabilities.canCopy && copyMoveTo.canPick(targetFiles)
		},
		{
			icon: ArrowBendUpRight,
			label: t('contextMenu.moveTo'),
			onClick: () => void copyMoveTo.moveTo(getTargetFiles()),
			condition: () => capabilities.canCopy && copyMoveTo.canPick(targetFiles)
		},
		{
			icon: Copy,
			label:
				selected && selectedFiles.length > 1
					? t('contextMenu.duplicateItems', {count: selectedFiles.length})
		: t('contextMenu.duplicate'),
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
			label: t('contextMenu.imageProcessing'),
			condition: () => !!file && getContentKind(file) === 'image',
			submenu: [
				{
					icon: Sparkle,
					label: t('contextMenu.generateBlurhash'),
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
					label: t('contextMenu.regenerateThumbnail'),
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
					label: t('contextMenu.extractText'),
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
			label: t('contextMenu.videoProcessing'),
			condition: () => !!file && getContentKind(file) === 'video',
			submenu: [
				{
					icon: FilmStrip,
					label: t('contextMenu.generateThumbstrip'),
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
					label: t('contextMenu.generateBlurhash'),
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
					label: t('contextMenu.regenerateThumbnail'),
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
					label: t('contextMenu.extractSubtitles'),
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
					label: t('contextMenu.generateProxy'),
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
			label: t('contextMenu.audioProcessing'),
			condition: () => !!file && getContentKind(file) === 'audio',
			submenu: [
				{
					icon: TextAa,
					label: t('contextMenu.transcribeAudio'),
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
			label: t('contextMenu.documentProcessing'),
			condition: () =>
				!!file &&
				file.kind === 'File' &&
				['pdf', 'doc', 'docx'].includes(file.extension || ''),
			submenu: [
				{
					icon: TextAa,
					label: t('contextMenu.extractText'),
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
					label: t('contextMenu.regenerateThumbnail'),
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
			label: t('contextMenu.processItems', {count: selectedFiles.length}),
			condition: () =>
				selected && selectedFiles.length > 1 && !hasVirtualFiles,
			submenu: [
				{
					icon: Crop,
					label: t('contextMenu.regenerateAllThumbnails'),
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
					label: t('contextMenu.generateBlurhashes'),
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
					label: t('contextMenu.extractText'),
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
					? t('contextMenu.removeTagItems', {count: selectedFiles.length})
		: t('contextMenu.removeTag'),
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
					toast.error(t('errors.failedToRemoveTag', {error: err}));
				}
			},
			condition: () => mode.type === 'tag' && !hasVirtualFiles
		},
		{type: 'separator'},
		{
			icon: Trash,
			label:
				selected && selectedFiles.length > 1
					? t('contextMenu.deleteItems', {count: selectedFiles.length})
					: t('contextMenu.delete'),
			onClick: async () => {
				const targets = getTargetFiles();
				await deleteFiles(targets, false);
			},
			keybindId: 'explorer.delete',
			variant: 'danger' as const,
			condition: () => capabilities.canDelete
		}
	];
	const menu = useContextMenu({items: inArchive ? archiveItems : items});
	return {...menu, appsLoading};
}
