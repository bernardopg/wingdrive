import type {File, WingPath} from '@wingdrive/ts-client';
import clsx from 'clsx';
import {useEffect, useId, useState} from 'react';
import {
	type FilePickerError,
	type FilePickerRequest,
	type FilePickerSelection,
	usePlatform
} from '../../contexts/PlatformContext';
import {useLibraryQuery} from '../../contexts/WingDriveContext';
import {useTranslation} from '../../i18n';
import {useExplorer} from '../../routes/explorer';
import {useSelection} from '../../routes/explorer/SelectionContext';
import {
	acceptLabel,
	buildSelection,
	initialChoices,
	initialFilter,
	type PickedEntry,
	type PickerProblem
} from './filePicker';

const buttonClass =
	'rounded-md px-3 py-1.5 text-sm font-medium transition-colors disabled:opacity-50';
const fieldClass =
	'rounded-md border border-app-line bg-app-input px-2 py-1 text-sm text-ink focus:outline-none focus:ring-1 focus:ring-accent';

function isFilePickerError(value: unknown): value is FilePickerError {
	return (
		typeof value === 'object' &&
		value !== null &&
		'kind' in value &&
		(value.kind === 'exists' || value.kind === 'invalid')
	);
}

/** Paths on this device; the picker returns only those, as `file://` URIs. */
function useLocalPath() {
	const {data: devices} = useLibraryQuery({
		type: 'devices.list',
		input: {include_offline: true, include_details: false}
	});
	const localSlug = devices?.find((device) => device.is_current)?.slug;
	return (path: WingPath | null | undefined): string | undefined => {
		if (!path || !('Physical' in path)) return undefined;
		const {device_slug, path: local} = path.Physical;
		return device_slug === 'local' || device_slug === localSlug ? local : undefined;
	};
}

/**
 * The confirm bar of a file chooser window: the Explorer above it browses,
 * this bar returns the choice to the app that asked through the portal.
 */
export function FilePickerBar() {
	const platform = usePlatform();
	const {t} = useTranslation('explorer');
	const [request, setRequest] = useState<FilePickerRequest | null | undefined>(undefined);

	useEffect(() => {
		let active = true;
		platform
			.getFilePickerRequest?.()
			.then((value) => active && setRequest(value))
			.catch(() => active && setRequest(null));
		return () => {
			active = false;
		};
	}, [platform]);

	if (request === undefined) return null;
	if (request === null) {
		return (
			<div className="flex items-center justify-between gap-3 border-t border-app-line bg-app-box px-4 py-3 text-sm text-ink-dull">
				<span>{t('filePicker.ended')}</span>
				<button
					type="button"
					className={clsx(buttonClass, 'bg-app-button text-ink hover:bg-app-hover')}
					onClick={() => void platform.closeCurrentWindow?.()}
				>
					{t('filePicker.close')}
				</button>
			</div>
		);
	}
	return <PickerControls request={request} />;
}

function PickerControls({request}: {request: FilePickerRequest}) {
	const platform = usePlatform();
	const {t} = useTranslation('explorer');
	const {currentPath, navigateToPath} = useExplorer();
	const {selectedFiles} = useSelection();
	const localPath = useLocalPath();
	const nameId = useId();
	const filterId = useId();

	const [name, setName] = useState(request.suggested_name ?? '');
	const [filter, setFilter] = useState(() => initialFilter(request));
	const [choices, setChoices] = useState(() => initialChoices(request));
	const [problem, setProblem] = useState<PickerProblem | null>(null);
	const [refusal, setRefusal] = useState<string | null>(null);
	const [confirmReplace, setConfirmReplace] = useState<FilePickerSelection | null>(null);
	const [busy, setBusy] = useState(false);

	const currentFolder = localPath(currentPath);
	const picked: PickedEntry[] = selectedFiles.map((file: File) => ({
		name: file.name,
		path: localPath(file.wing_path),
		isDirectory: file.kind === 'Directory'
	}));

	const {mode, options} = request;
	const accept =
		acceptLabel(options.accept_label) ??
		(mode === 'open'
			? options.directory
				? t('filePicker.selectFolder')
				: t('filePicker.open')
			: mode === 'save'
				? t('filePicker.save')
				: t('filePicker.saveHere'));

	const send = async (selection: FilePickerSelection) => {
		if (!platform.respondFilePicker) return;
		setBusy(true);
		try {
			await platform.respondFilePicker(selection);
		} catch (error) {
			if (isFilePickerError(error) && error.kind === 'exists' && selection.kind === 'save') {
				setConfirmReplace({...selection, overwrite: true});
			} else {
				setRefusal(isFilePickerError(error) && error.kind === 'invalid' ? error.reason : String(error));
			}
		} finally {
			setBusy(false);
		}
	};

	const submit = () => {
		setProblem(null);
		setRefusal(null);
		setConfirmReplace(null);
		// Open on a single folder while picking files goes into it, as other file choosers do.
		if (mode === 'open' && !options.directory && selectedFiles.length === 1) {
			const [only] = selectedFiles;
			if (only && only.kind === 'Directory') {
				navigateToPath(only.wing_path);
				return;
			}
		}
		const result = buildSelection(request, {
			picked,
			currentFolder,
			name,
			filter,
			choices
		});
		if (!result.ok) {
			setProblem(result.problem);
			return;
		}
		void send(result.selection);
	};

	const cancel = () => {
		void platform.cancelFilePicker?.();
	};

	const message = confirmReplace
		? null
		: refusal ?? (problem ? t(`filePicker.errors.${problem}`) : null);

	return (
		<div
			className="flex flex-col gap-2 border-t border-app-line bg-app-box px-4 py-3"
			role="group"
			aria-label={request.title || t('filePicker.title')}
		>
			{mode === 'save' && (
				<div className="flex items-center gap-2">
					<label htmlFor={nameId} className="w-20 shrink-0 text-sm text-ink-dull">
						{t('filePicker.name')}
					</label>
					<input
						id={nameId}
						type="text"
						autoFocus
						value={name}
						onChange={(event) => {
							setName(event.target.value);
							setConfirmReplace(null);
						}}
						onFocus={(event) => {
							// Select the name without its extension, as GTK does.
							const dot = event.target.value.lastIndexOf('.');
							event.target.setSelectionRange(0, dot > 0 ? dot : event.target.value.length);
						}}
						onKeyDown={(event) => {
							if (event.key === 'Enter') submit();
						}}
						className={clsx(fieldClass, 'min-w-0 flex-1')}
					/>
				</div>
			)}
			{mode === 'save_files' && options.files.length > 0 && (
				<p className="truncate text-xs text-ink-dull" title={options.files.join(', ')}>
					{t('filePicker.savingFiles', {count: options.files.length})}
				</p>
			)}
			<div className="flex flex-wrap items-center gap-2">
				{options.filters.length > 0 && filter !== null && (
					<>
						<label htmlFor={filterId} className="sr-only">
							{t('filePicker.filter')}
						</label>
						<select
							id={filterId}
							value={filter}
							onChange={(event) => setFilter(Number(event.target.value))}
							className={fieldClass}
						>
							{options.filters.map((entry, index) => (
								<option key={`${entry.name}-${index}`} value={index}>
									{entry.name}
								</option>
							))}
						</select>
					</>
				)}
				{options.choices.map((choice) =>
					choice.options.length === 0 ? (
						<label key={choice.id} className="flex items-center gap-1.5 text-sm text-ink">
							<input
								type="checkbox"
								checked={choices[choice.id] === 'true'}
								onChange={(event) =>
									setChoices({...choices, [choice.id]: event.target.checked ? 'true' : 'false'})
								}
							/>
							{choice.label}
						</label>
					) : (
						<label key={choice.id} className="flex items-center gap-1.5 text-sm text-ink-dull">
							{choice.label}
							<select
								value={choices[choice.id]}
								onChange={(event) => setChoices({...choices, [choice.id]: event.target.value})}
								className={fieldClass}
							>
								{choice.options.map((option) => (
									<option key={option.id} value={option.id}>
										{option.label}
									</option>
								))}
							</select>
						</label>
					)
				)}
				<div className="min-w-0 flex-1 truncate text-sm" role="status">
					{confirmReplace && confirmReplace.kind === 'save' ? (
						<span className="text-status-warning">
							{t('filePicker.replaceQuestion', {
								name: confirmReplace.path.split('/').pop()
							})}
						</span>
					) : message ? (
						<span className="text-status-error">{message}</span>
					) : !currentFolder && mode !== 'open' ? (
						<span className="text-ink-dull">{t('filePicker.errors.notLocal')}</span>
					) : null}
				</div>
				<button
					type="button"
					onClick={cancel}
					disabled={busy}
					className={clsx(buttonClass, 'bg-app-button text-ink hover:bg-app-hover')}
				>
					{t('filePicker.cancel')}
				</button>
				{confirmReplace ? (
					<button
						type="button"
						onClick={() => void send(confirmReplace)}
						disabled={busy}
						className={clsx(buttonClass, 'bg-status-error text-white hover:bg-status-error/80')}
					>
						{t('filePicker.replace')}
					</button>
				) : (
					<button
						type="button"
						onClick={submit}
						disabled={busy}
						className={clsx(buttonClass, 'bg-accent text-white hover:bg-accent-deep')}
					>
						{accept}
					</button>
				)}
			</div>
		</div>
	);
}
