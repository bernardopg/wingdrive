import {
	ArrowRight,
	ArrowsLeftRight,
	CircleNotch,
	Copy as CopyIcon,
	Files,
	FolderOpen,
	Warning
} from '@phosphor-icons/react';
import {
	Dialog,
	dialogManager,
	useDialog,
	type UseDialogProps
} from '@wingdrive/primitives';
import type {File as FileType, WingPath} from '@wingdrive/ts-client';
import {useEffect, useState} from 'react';
import {useForm} from 'react-hook-form';
import {usePlatform} from '../../contexts/PlatformContext';
import {
	useLibraryMutation,
	useLibraryQuery
} from '../../contexts/WingDriveContext';
import {
	summarizeFileOperation,
	type FileOperationOutcome
} from '../../hooks/fileOperationOutcome';
import {useRefetchFileListings} from '../../hooks/useRefetchFileListings';
import {useUndo} from '../../hooks/useUndo';
import {useWaitForJob} from '../../hooks/useWaitForJob';
import {File, FileStack} from '../../routes/explorer/File';

interface FileOperationDialogProps {
	id: number;
	operation: 'copy' | 'move';
	sources: WingPath[];
	destination: WingPath;
	onComplete?: () => void;
}

type ConflictResolution = 'Overwrite' | 'AutoModifyName' | 'Skip' | 'Abort';

type DialogPhase =
	| {type: 'form'}
	| {type: 'executing'}
	| {type: 'error'; message: string}
	| {type: 'outcome'; outcome: FileOperationOutcome};

export function useFileOperationDialog() {
	return (options: Omit<FileOperationDialogProps, 'id'>) => {
		return dialogManager.create((props: UseDialogProps) => (
			<FileOperationDialog
				{...(props as FileOperationDialogProps)}
				{...options}
			/>
		));
	};
}

function FileOperationDialog(props: FileOperationDialogProps) {
	const dialog = useDialog(props);
	const form = useForm();
	const [phase, setPhase] = useState<DialogPhase>({type: 'form'});
	const [operation, setOperation] = useState<'copy' | 'move'>(
		props.operation
	);
	const [conflictResolution, setConflictResolution] =
		useState<ConflictResolution>('Skip');

	const copyFiles = useLibraryMutation('files.copy');
	const waitForJob = useWaitForJob();
	const refetchListings = useRefetchFileListings();
	const platform = usePlatform();
	const undo = useUndo();

	// Fixed-length slots keep the hook count constant (Rules of Hooks): mapping
	// over a variable number of sources made React reorder query state whenever
	// the selection size changed while the dialog was open.
	const sourceSlots = Array.from({length: 3}, (_, i) => {
		const source = props.sources[i];
		return source && 'Physical' in source ? source.Physical.path : null;
	});

	const sourceFileQueries = sourceSlots.map((path) =>
		useLibraryQuery(
			{type: 'files.by_path', input: {path: path ?? ''}},
			{enabled: !!path}
		)
	);

	const sourceFiles = sourceFileQueries
		.map((q) => q.data)
		.filter((f): f is FileType => f !== undefined && f !== null);

	// Fetch destination folder info
	const destPath: string | null =
		'Physical' in props.destination
			? props.destination.Physical.path
			: null;

	const {data: destFile} = useLibraryQuery(
		{type: 'files.by_path', input: {path: destPath!}},
		{enabled: !!destPath}
	);

	// Check if any source is the same as destination
	const hasSameSourceDest = props.sources.some((source) => {
		if ('Physical' in source && 'Physical' in props.destination) {
			return source.Physical.path === props.destination.Physical.path;
		}
		return false;
	});

	// Auto-close if invalid operation (must be in useEffect to avoid render loop)
	useEffect(() => {
		if (hasSameSourceDest) {
			dialogManager.setState(props.id, {open: false});
		}
	}, [hasSameSourceDest, props.id]);

	if (hasSameSourceDest) {
		return null;
	}

	const handleSubmit = async () => {
		try {
			setPhase({type: 'executing'});
			const move = platform.undoMove;
			const identify = platform.fileIdentity;
			const candidates =
				operation === 'move' &&
				conflictResolution === 'Skip' &&
				move &&
				identify &&
				undo.isLocalPath(props.destination) &&
				'Physical' in props.destination
					? await Promise.all(
							props.sources.flatMap((source) => {
								if (
									!('Physical' in source) ||
									!undo.isLocalPath(source) ||
									!('Physical' in props.destination)
								)
									return [];
								const oldPath = source.Physical.path;
								const name = oldPath.slice(
									Math.max(
										oldPath.lastIndexOf('/'),
										oldPath.lastIndexOf('\\')
									) + 1
								);
								const newPath =
									props.destination.Physical.path.replace(
										/[\\/]$/,
										''
									) +
									'/' +
									name;
								return [
									identify(oldPath)
										.then((expected) => ({
											oldPath,
											newPath,
											expected
										}))
										.catch(() => null)
								];
							})
						)
					: [];

			// Execute with the user's chosen operation and conflict resolution
			// The mutation only queues the job; wait for it to finish so the
			// listing refresh below actually sees the new files on disk.
			const {result} = await waitForJob(() =>
				copyFiles.mutateAsync({
					sources: {paths: props.sources},
					destination: props.destination,
					overwrite: conflictResolution === 'Overwrite',
					verify_checksum: false,
					preserve_timestamps: true,
					move_files: operation === 'move',
					copy_method: 'Auto',
					on_conflict: conflictResolution
				})
			);
			refetchListings();

			const outcome = summarizeFileOperation(operation, result);
			// Items that did move can be undone even when others failed; the
			// identity check skips the ones that never arrived.
			if (
				move &&
				identify &&
				(outcome.status === 'success' || outcome.status === 'partial')
			) {
				for (const candidate of candidates) {
					if (!candidate) continue;
					try {
						const current = await identify(candidate.newPath);
						if (
							JSON.stringify(current) ===
							JSON.stringify(candidate.expected)
						) {
							undo.record('move', () =>
								move(
									candidate.newPath,
									candidate.oldPath,
									candidate.expected
								)
							);
						}
					} catch {
						/* A skipped or cross-volume item has no safe inode-preserving inverse. */
					}
				}
			}

			if (outcome.status !== 'success') {
				// The clipboard keeps a cut until every item arrived, so the
				// user can paste again to retry the ones that failed.
				setPhase({type: 'outcome', outcome});
				return;
			}

			dialogManager.setState(props.id, {open: false});
			props.onComplete?.();
		} catch (error) {
			setPhase({
				type: 'error',
				message:
					error instanceof Error ? error.message : 'Operation failed'
			});
		}
	};

	const handleCancel = () => {
		dialogManager.setState(props.id, {open: false});
	};

	// Keyboard shortcuts
	useEffect(() => {
		if (phase.type !== 'form') return;

		const handleKeyDown = (e: KeyboardEvent) => {
			// Enter - Submit
			if (e.key === 'Enter' && !e.shiftKey) {
				e.preventDefault();
				handleSubmit();
				return;
			}

			// Only handle other shortcuts if not typing in an input
			if ((e.target as HTMLElement)?.tagName === 'INPUT') return;

			// ⌘1 / Ctrl+1 - Copy mode
			if ((e.metaKey || e.ctrlKey) && e.key === '1') {
				e.preventDefault();
				e.stopPropagation();
				setOperation('copy');
			}
			// ⌘2 / Ctrl+2 - Move mode
			if ((e.metaKey || e.ctrlKey) && e.key === '2') {
				e.preventDefault();
				e.stopPropagation();
				setOperation('move');
			}
			// S - Skip
			if (e.key === 's' && !e.metaKey && !e.ctrlKey) {
				e.preventDefault();
				setConflictResolution('Skip');
			}
			// K - Keep both
			if (e.key === 'k' && !e.metaKey && !e.ctrlKey) {
				e.preventDefault();
				setConflictResolution('AutoModifyName');
			}
			// O - Overwrite
			if (e.key === 'o' && !e.metaKey && !e.ctrlKey) {
				e.preventDefault();
				setConflictResolution('Overwrite');
			}
		};

		window.addEventListener('keydown', handleKeyDown);
		return () => window.removeEventListener('keydown', handleKeyDown);
	}, [phase.type, operation, conflictResolution]);

	// Executing state
	if (phase.type === 'executing') {
		return (
			<Dialog
				dialog={dialog}
				form={form}
				title={operation === 'copy' ? 'Copying Files' : 'Moving Files'}
				icon={<Files size={20} weight="bold" />}
				hideButtons
			>
				<div className="space-y-3 py-6">
					<div className="flex items-center justify-center gap-3">
						<CircleNotch
							className="text-accent size-6 animate-spin"
							weight="bold"
						/>
						<span className="text-ink text-sm">
							{operation === 'copy'
								? 'Copying files...'
								: 'Moving files...'}
						</span>
					</div>
				</div>
			</Dialog>
		);
	}

	if (phase.type === 'outcome') {
		const {outcome} = phase;
		const isError = outcome.status === 'failed';
		const title =
			outcome.status === 'partial'
				? 'Finished with Problems'
				: outcome.status === 'cancelled'
					? 'Operation Cancelled'
					: outcome.status === 'running'
						? 'Still Running'
						: 'Operation Failed';
		return (
			<Dialog
				dialog={dialog}
				form={form}
				title={title}
				icon={
					<Warning
						size={20}
						weight="fill"
						className={isError ? 'text-red-500' : 'text-amber-500'}
					/>
				}
				ctaLabel="Close"
				onCancelled={false}
				onSubmit={form.handleSubmit(handleCancel)}
			>
				<div className="flex flex-col gap-3 py-4">
					<p className="text-ink text-sm" role="status">
						{outcome.message}
					</p>
					{outcome.details.length > 0 && (
						<ul className="border-app-line bg-app-box text-ink-dull max-h-40 space-y-1 overflow-y-auto rounded-md border p-2 text-xs">
							{outcome.details.map((detail) => (
								<li key={detail} className="break-all">
									{detail}
								</li>
							))}
						</ul>
					)}
					{outcome.failed > outcome.details.length && (
						<p className="text-ink-faint text-xs">
							The job log lists every failed item.
						</p>
					)}
				</div>
			</Dialog>
		);
	}

	// Error state
	if (phase.type === 'error') {
		return (
			<Dialog
				dialog={dialog}
				form={form}
				title="Operation Failed"
				icon={
					<Warning size={20} weight="fill" className="text-red-500" />
				}
				ctaLabel="Close"
				onSubmit={form.handleSubmit(handleCancel)}
			>
				<div className="flex flex-col gap-4 py-4">
					<div className="flex items-start gap-2 rounded-md border border-red-500/20 bg-red-500/10 p-3">
						<Warning
							className="mt-0.5 size-5 text-red-500"
							weight="fill"
						/>
						<div className="flex-1">
							<div className="text-ink mb-1 text-sm font-medium">
								Error
							</div>
							<div className="text-ink-dull text-xs">
								{phase.message}
							</div>
						</div>
					</div>
				</div>
			</Dialog>
		);
	}

	const sourceCount = props.sources.length;
	const pluralItems = sourceCount === 1 ? 'item' : 'items';

	// Form state - let user choose operation and conflict resolution
	return (
		<Dialog
			dialog={dialog}
			form={form}
			title="File Operation"
			icon={<Files size={20} weight="bold" />}
			ctaLabel={operation === 'copy' ? 'Copy' : 'Move'}
			onSubmit={form.handleSubmit(handleSubmit)}
			onCancelled={handleCancel}
			formClassName="!min-w-[400px] !max-w-[400px]"
		>
			<div className="space-y-5 py-2">
				{/* Source → Destination visual */}
				<div className="flex items-center gap-4">
					{/* Source */}
					<div className="flex min-w-0 flex-1 flex-col items-center gap-2">
						{sourceFiles.length > 0 ? (
							<>
								{sourceFiles.length === 1 ? (
									<File.Thumb
										file={sourceFiles[0]}
										size={80}
									/>
								) : (
									<FileStack files={sourceFiles} size={80} />
								)}
								<div className="w-full text-center">
									<div className="text-ink-dull mb-0.5 text-xs">
										Source
									</div>
									{sourceFiles.length === 1 ? (
										<div className="text-ink w-full truncate text-sm font-medium">
											{sourceFiles[0].name}
										</div>
									) : (
										<div className="text-ink text-sm font-medium">
											{sourceCount} {pluralItems}
										</div>
									)}
								</div>
							</>
						) : (
							<>
								<Files
									className="text-ink-dull size-20"
									weight="fill"
								/>
								<div className="text-center">
									<div className="text-ink-dull mb-0.5 text-xs">
										Source
									</div>
									<div className="text-ink text-sm font-medium">
										{sourceCount} {pluralItems}
									</div>
								</div>
							</>
						)}
					</div>

					{/* Arrow */}
					<div className="flex-shrink-0">
						<ArrowRight
							className="text-accent size-6"
							weight="bold"
						/>
					</div>

					{/* Destination */}
					<div className="flex min-w-0 flex-1 flex-col items-center gap-2">
						{destFile ? (
							<>
								<File.Thumb file={destFile} size={80} />
								<div className="w-full text-center">
									<div className="text-ink-dull mb-0.5 text-xs">
										To
									</div>
									<div className="text-ink w-full truncate text-sm font-medium">
										{destFile.name}
									</div>
								</div>
							</>
						) : (
							<>
								<FolderOpen
									className="text-accent size-20"
									weight="fill"
								/>
								<div className="text-center">
									<div className="text-ink-dull mb-0.5 text-xs">
										To
									</div>
									<div className="text-ink max-w-full truncate text-sm font-medium">
										{getFileName(props.destination)}
									</div>
								</div>
							</>
						)}
					</div>
				</div>

				{/* Operation type selection */}
				<div className="space-y-2">
					<div className="text-ink-dull mb-2 text-xs font-medium">
						Operation:
					</div>
					<div className="flex gap-2">
						<button
							type="button"
							onClick={() => setOperation('copy')}
							className={`flex flex-1 items-center justify-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors ${
								operation === 'copy'
									? 'bg-accent text-white'
									: 'bg-app-box text-ink hover:bg-app-hover'
							}`}
						>
							<CopyIcon className="size-4" weight="bold" />
							Copy
							<span className="text-xs opacity-60">⌘1</span>
						</button>
						<button
							type="button"
							onClick={() => setOperation('move')}
							className={`flex flex-1 items-center justify-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors ${
								operation === 'move'
									? 'bg-accent text-white'
									: 'bg-app-box text-ink hover:bg-app-hover'
							}`}
						>
							<ArrowsLeftRight className="size-4" weight="bold" />
							Move
							<span className="text-xs opacity-60">⌘2</span>
						</button>
					</div>
				</div>

				{/* Conflict resolution options */}
				<div className="space-y-2">
					<div className="text-ink-dull mb-2 text-xs font-medium">
						If files already exist:
					</div>
					<div className="space-y-1">
						{[
							{
								value: 'Skip',
								label: 'Skip existing files',
								key: 'S'
							},
							{
								value: 'AutoModifyName',
								label: 'Keep both (rename new files)',
								key: 'K'
							},
							{
								value: 'Overwrite',
								label: 'Overwrite existing files',
								key: 'O'
							}
						].map((option) => (
							<label
								key={option.value}
								className="hover:bg-app-hover flex cursor-pointer items-center justify-between gap-2 rounded-md px-2 py-2 transition-colors"
							>
								<div className="flex items-center gap-2">
									<input
										type="radio"
										name="conflict-resolution"
										value={option.value}
										checked={
											conflictResolution === option.value
										}
										onChange={() =>
											setConflictResolution(
												option.value as ConflictResolution
											)
										}
										className="accent-accent size-4 cursor-pointer"
									/>
									<span className="text-ink text-sm">
										{option.label}
									</span>
								</div>
								<span className="text-ink-faint text-xs font-medium">
									{option.key}
								</span>
							</label>
						))}
					</div>
				</div>
			</div>
		</Dialog>
	);
}

// Utility functions
function getFileName(path: WingPath): string {
	if (!path || typeof path !== 'object') {
		return 'Unknown';
	}

	if ('Physical' in path && path.Physical) {
		const pathStr = path.Physical.path || '';
		const parts = pathStr.split('/');
		return parts[parts.length - 1] || pathStr;
	}

	if ('Cloud' in path && path.Cloud) {
		const pathStr = path.Cloud.path || '';
		const parts = pathStr.split('/');
		return parts[parts.length - 1] || pathStr;
	}

	return 'Unknown';
}
