import {ArrowLeft} from '@phosphor-icons/react';
import {CircleButton} from '@wingdrive/primitives';
import {useCallback, useMemo, useState} from 'react';
import {useNavigate} from 'react-router-dom';
import {SourceTypeIcon} from '../../components/Sources/SourceTypeIcon';
import {
	useLibraryMutation,
	useLibraryQuery
} from '../../contexts/WingDriveContext';
import {TopBarItem, TopBarPortal} from '../../TopBar';
import {ExpandableSearchButton} from '../explorer/components/ExpandableSearchButton';
import {useTranslation} from '../../i18n';

export function AdaptersScreen() {
	const {t} = useTranslation('navigation');
	const navigate = useNavigate();
	const [searchValue, setSearchValue] = useState('');

	const {data: adaptersRaw, isLoading} = useLibraryQuery({
		type: 'adapters.list',
		input: {}
	});
	const adapters = adaptersRaw as any[] | undefined;

	const [selectedAdapter, setSelectedAdapter] = useState<string | null>(null);
	const selected = adapters?.find((a) => a.id === selectedAdapter);

	const filteredAdapters = useMemo(() => {
		if (!adapters || !searchValue.trim()) return adapters;
		const q = searchValue.toLowerCase();
		return adapters.filter(
			(a) =>
				a.name.toLowerCase().includes(q) ||
				a.description.toLowerCase().includes(q) ||
				a.data_type.toLowerCase().includes(q)
		);
	}, [adapters, searchValue]);

	return (
		<>
			<TopBarPortal
				left={
					<>
						<TopBarItem
							id="navigation"
							label={t('common.navigation')}
							priority="high"
						>
							<CircleButton
								icon={ArrowLeft}
								onClick={() => navigate(-1)}
							/>
						</TopBarItem>
						<TopBarItem id="title" label={t('common.title')} priority="high">
							<span className="text-ink whitespace-nowrap text-xs font-medium">
								{t('sources.adaptersTitle')}
							</span>
						</TopBarItem>
					</>
				}
				right={
					<>
						<TopBarItem id="search" label={t('common.search')} priority="high">
							<ExpandableSearchButton
								placeholder={t('sources.searchAdapters')}
								value={searchValue}
								onChange={setSearchValue}
								onClear={() => setSearchValue('')}
							/>
						</TopBarItem>
					</>
				}
			/>
			<div className="p-6">
				{isLoading && (
					<div className="flex items-center justify-center py-20">
						<div className="text-ink-faint text-sm">{t('common.loading')}</div>
					</div>
				)}

				{adapters && adapters.length === 0 && !searchValue && (
					<div className="flex flex-col items-center justify-center py-20">
						<p className="text-ink-dull text-sm">
							{t('sources.noAdapters')}
						</p>
						<p className="text-ink-faint mt-1 text-xs">
							{t('sources.adaptersHint')}
						</p>
					</div>
				)}

				{filteredAdapters &&
					filteredAdapters.length === 0 &&
					searchValue && (
						<div className="flex items-center justify-center py-20">
							<p className="text-ink-faint text-sm">
								{t('sources.noMatchingAdapters')}
							</p>
						</div>
					)}

				{filteredAdapters && filteredAdapters.length > 0 && (
					<div className="grid grid-cols-2 gap-3 lg:grid-cols-3">
						{filteredAdapters.map((adapter) => (
							<button
								key={adapter.id}
								onClick={() => setSelectedAdapter(adapter.id)}
								className="border-app-line bg-app-box hover:border-app-line/80 hover:bg-app-hover flex items-center gap-3 rounded-lg border p-3 text-left transition-all"
							>
								<SourceTypeIcon
									type={adapter.data_type}
									svg={adapter.icon_svg}
									size="md"
								/>
								<div className="min-w-0 flex-1">
									<div className="flex items-center gap-2">
										<span className="text-ink truncate text-sm font-medium">
											{adapter.name}
										</span>
										<span className="bg-app-line/60 text-ink-faint shrink-0 rounded px-1.5 py-0.5 text-[10px]">
											v{adapter.version}
										</span>
										{adapter.update_available && (
											<span className="shrink-0 rounded-full bg-blue-500/15 px-2 py-0.5 text-[10px] font-medium text-blue-400 ring-1 ring-blue-500/30">
												{t('sources.updateAvailable')}
											</span>
										)}
									</div>
									<div className="text-ink-faint mt-0.5 truncate text-xs">
										{adapter.description ||
											adapter.data_type}
									</div>
									{adapter.author && (
										<div className="text-ink-faint/70 mt-0.5 text-[11px]">
											{t('sources.byAuthor', {author: adapter.author})}
										</div>
									)}
								</div>
							</button>
						))}
					</div>
				)}

				{/* Configure dialog */}
				{selected && (
					<div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
						<div className="border-app-line bg-app-box w-full max-w-md rounded-xl border p-6 shadow-2xl">
							<div className="mb-4 flex items-center gap-3">
								<SourceTypeIcon
									type={selected.data_type}
									svg={selected.icon_svg}
									size="sm"
								/>
								<h3 className="text-ink text-sm font-semibold">
									{selected.name}
								</h3>
							</div>
							<ConfigureForm
								adapterId={selected.id}
								adapterName={selected.name}
								onDone={() => setSelectedAdapter(null)}
							/>
						</div>
					</div>
				)}
			</div>
		</>
	);
}

function ConfigureForm({
	adapterId,
	adapterName,
	onDone
}: {
	adapterId: string;
	adapterName: string;
	onDone: () => void;
}) {
	const {t} = useTranslation('navigation');
	const navigate = useNavigate();

	const {
		data: fields,
		isLoading,
		error: queryError
	} = useLibraryQuery({
		type: 'adapters.config',
		input: {adapter_id: adapterId}
	});

	const createSource = useLibraryMutation('sources.create');
	const syncSource = useLibraryMutation('sources.sync');

	const [name, setName] = useState('');
	const [values, setValues] = useState<Record<string, string>>({});
	const [error, setError] = useState<string | null>(null);
	const [result, setResult] = useState<string | null>(null);

	const handleFieldChange = useCallback((key: string, value: string) => {
		setValues((prev) => ({...prev, [key]: value}));
	}, []);

	const handleSubmit = async () => {
		if (!name.trim()) {
			setError(t('sources.nameRequired'));
			return;
		}

		try {
			const config: Record<string, unknown> = {};
			for (const field of fields ?? []) {
				const val = values[field.key]?.trim() ?? '';
				if (field.required && !val) {
					setError(t('sources.fieldRequired', {name: field.name}));
					return;
				}
				if (!val && !field.required) continue;

				if (field.field_type === 'integer' && val) {
					config[field.key] = parseInt(val, 10);
				} else if (field.field_type === 'boolean') {
					config[field.key] = val === 'true';
				} else {
					config[field.key] = val;
				}
			}

			const source = await createSource.mutateAsync({
				name: name.trim(),
				adapter_id: adapterId,
				config: config as any
			});

			// Dispatch sync as a background job
			await syncSource.mutateAsync({
				source_id: source.id
			});

			setResult(
				t('sources.created')
			);
		} catch (e) {
			setError(String(e));
		}
	};

	if (isLoading) {
		return (
				<div className="text-ink-faint py-8 text-center text-sm">
					{t('common.loading')}
			</div>
		);
	}

	if (queryError) {
		return (
			<div className="rounded-md border border-red-400/20 p-3 text-xs text-red-400">
				{t('sources.configLoadFailed', {error: String(queryError)})}
			</div>
		);
	}

	if (result) {
		const isError = result.startsWith('Sync failed');
		return (
			<div className="flex flex-col items-center py-6">
				<p
					className={`text-sm ${isError ? 'text-red-400' : 'text-ink'}`}
				>
					{isError ? t('common.somethingWentWrong') : t('sources.added')}
				</p>
				<p className="text-ink-faint mt-1 text-xs">{result}</p>
				<button
					onClick={() => {
						onDone();
						navigate('/sources');
					}}
					className="bg-accent hover:bg-accent-deep mt-4 rounded-lg px-3.5 py-1.5 text-sm font-medium text-white transition-colors"
				>
					{t('common.done')}
				</button>
			</div>
		);
	}

	return (
		<div className="flex flex-col gap-3">
			<div>
				<label className="text-ink-dull mb-1 block text-xs font-medium">
					{t('sources.name')}
				</label>
				<input
					value={name}
					onChange={(e) => setName(e.target.value)}
					placeholder={t('sources.nameExample', {name: adapterName})}
					className="border-app-line bg-app-input text-ink focus:ring-accent w-full rounded-md border px-3 py-2 text-sm focus:outline-none focus:ring-1"
				/>
			</div>

			{fields?.map((field) => (
				<div key={field.key}>
					<label className="text-ink-dull mb-1 block text-xs font-medium">
						{field.name}
						{field.required && (
							<span className="ml-1 text-red-400">*</span>
						)}
					</label>
					{field.description && (
						<p className="text-ink-faint mb-1 text-[11px] leading-relaxed">
							{field.description}
						</p>
					)}
					<input
						type={field.secret ? 'password' : 'text'}
						value={values[field.key] ?? ''}
						onChange={(e) =>
							handleFieldChange(field.key, e.target.value)
						}
						placeholder={
							field.default
								? t('sources.defaultValue', {value: field.default})
								: undefined
						}
						className="border-app-line bg-app-input text-ink focus:ring-accent w-full rounded-md border px-3 py-2 text-sm focus:outline-none focus:ring-1"
					/>
				</div>
			))}

			{error && (
				<div className="rounded-md border border-red-400/20 p-2 text-xs text-red-400">
					{error}
				</div>
			)}

			<div className="mt-1 flex gap-2">
				<button
					onClick={handleSubmit}
					disabled={createSource.isPending}
					className="bg-accent hover:bg-accent-deep flex-1 rounded-lg px-3.5 py-1.5 text-sm font-medium text-white transition-colors disabled:opacity-40"
				>
					{t('sources.addSync')}
				</button>
				<button
					onClick={onDone}
					className="text-ink-faint hover:text-ink rounded-lg px-3.5 py-1.5 text-sm font-medium transition-colors"
				>
					{t('common.cancel')}
				</button>
			</div>
		</div>
	);
}
