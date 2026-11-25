import {
	ArrowClockwise,
	ArrowLeft,
	CheckCircle,
	FolderOpen,
	Globe,
	HardDrives,
	Key,
	SealCheck,
	ShieldCheck,
	ShieldWarning,
	Terminal,
	Warning
} from '@phosphor-icons/react';
import {useQueryClient} from '@tanstack/react-query';
import {CircleButton} from '@wingdrive/primitives';
import type {StoreCard} from '@wingdrive/ts-client';
import {useCallback, useEffect, useRef, useState} from 'react';
import {useNavigate, useSearchParams} from 'react-router-dom';
import {SourceTypeIcon} from '../../components/Sources/SourceTypeIcon';
import {usePlatform} from '../../contexts/PlatformContext';
import {
	useCoreMutation,
	useCoreQuery,
	useLibraryMutation,
	useLibraryQuery
} from '../../contexts/WingDriveContext';
import {useTranslation} from '../../i18n';
import {TopBarItem, TopBarPortal} from '../../TopBar';
import {ExpandableSearchButton} from '../explorer/components/ExpandableSearchButton';

type Tab = 'store' | 'installed';

/** The card's summary and description in the interface language, falling back to English. */
function useStoreText(card: StoreCard) {
	const {i18n} = useTranslation('navigation');
	const language = (i18n.resolvedLanguage ?? i18n.language ?? 'en').split('-')[0];
	const translated = card.translations?.[language];
	return {
		summary: translated?.summary ?? card.summary,
		description: translated?.description ?? card.description
	};
}

/** The adapter store: cards for every published adapter, and the ones installed here. */
export function AdaptersScreen() {
	const {t} = useTranslation('navigation');
	const navigate = useNavigate();
	const queryClient = useQueryClient();
	const [params, setParams] = useSearchParams();
	const [searchValue, setSearchValue] = useState('');
	const [tab, setTab] = useState<Tab>('store');

	const listing = useCoreQuery({type: 'adapters.store.list', input: {}});
	const refresh = useCoreMutation('adapters.store.refresh');
	const {data: loaded} = useLibraryQuery({type: 'adapters.list', input: {}});

	const refreshStore = useCallback(async () => {
		try {
			await refresh.mutateAsync({});
		} finally {
			await queryClient.invalidateQueries();
		}
	}, [refresh, queryClient]);

	// Fetch the index once when the store has never been loaded on this device.
	const firstLoad = useRef(false);
	useEffect(() => {
		if (listing.data && listing.data.sequence === null && !firstLoad.current) {
			firstLoad.current = true;
			void refreshStore();
		}
	}, [listing.data, refreshStore]);

	const cards = listing.data?.adapters ?? [];
	const installedCards = cards.filter((card) => card.installed_version);
	const devAdapters = (loaded ?? []).filter(
		(adapter) => !cards.some((card) => card.id === adapter.id && card.installed_version)
	);

	const query = searchValue.trim().toLowerCase();
	const matches = (card: StoreCard) =>
		!query ||
		[
			card.name,
			card.summary,
			card.author,
			...card.categories,
			...Object.values(card.translations ?? {}).map((text) => text.summary)
		].some((value) => value.toLowerCase().includes(query));
	const shown = (tab === 'store' ? cards : installedCards).filter(matches);

	const openId = params.get('adapter');
	const openCard = cards.find((card) => card.id === openId);
	const [configureId, setConfigureId] = useState<string | null>(null);
	const configure = (loaded ?? []).find((adapter) => adapter.id === configureId);

	const close = () => {
		params.delete('adapter');
		setParams(params, {replace: true});
	};

	return (
		<>
			<TopBarPortal
				left={
					<>
						<TopBarItem id="navigation" label={t('common.navigation')} priority="high">
							<CircleButton icon={ArrowLeft} onClick={() => navigate(-1)} />
						</TopBarItem>
						<TopBarItem id="title" label={t('common.title')} priority="high">
							<span className="text-ink whitespace-nowrap text-xs font-medium">
								{t('store.title')}
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
						<TopBarItem id="refresh" label={t('store.refresh')} priority="high">
							<CircleButton
								icon={ArrowClockwise}
								title={t('store.refresh')}
								disabled={refresh.isPending}
								onClick={() => void refreshStore()}
							/>
						</TopBarItem>
					</>
				}
			/>
			<div className="p-6">
				<div className="mb-5 flex items-center gap-1">
					<TabButton active={tab === 'store'} onClick={() => setTab('store')}>
						{t('store.tabStore')}
					</TabButton>
					<TabButton active={tab === 'installed'} onClick={() => setTab('installed')}>
						{t('store.tabInstalled')}
						{(listing.data?.updates ?? 0) > 0 && (
							<span className="ml-1.5 rounded-full bg-blue-500 px-1.5 text-[10px] font-semibold text-white">
								{listing.data?.updates}
							</span>
						)}
					</TabButton>
					<span className="text-ink-faint ml-auto text-[11px]">
						{refresh.isPending
							? t('store.refreshing')
							: listing.data?.sequence != null
								? t('store.indexVersion', {sequence: listing.data.sequence})
								: null}
					</span>
				</div>

				{refresh.error && (
					<div className="mb-4 rounded-md border border-red-400/20 p-3 text-xs text-red-400">
						{t('store.refreshFailed', {error: String(refresh.error)})}
					</div>
				)}

				{listing.isLoading && (
					<div className="text-ink-faint py-20 text-center text-sm">{t('common.loading')}</div>
				)}

				{!listing.isLoading && shown.length === 0 && (
					<div className="text-ink-faint py-20 text-center text-sm">
						{query
							? t('sources.noMatchingAdapters')
							: tab === 'store'
								? t('store.empty')
								: t('sources.noAdapters')}
					</div>
				)}

				{shown.length > 0 && (
					<div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
						{shown.map((card) => (
							<AdapterCard
								key={card.id}
								card={card}
								onOpen={() => setParams({adapter: card.id})}
							/>
						))}
					</div>
				)}

				{tab === 'installed' && devAdapters.length > 0 && (
					<div className="mt-8">
						<h2 className="text-ink-dull mb-2 text-xs font-medium">{t('store.local')}</h2>
						<div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
							{devAdapters.map((adapter) => (
								<button
									key={adapter.id}
									onClick={() => setConfigureId(adapter.id)}
									className="border-app-line bg-app-box hover:bg-app-hover flex items-center gap-3 rounded-xl border p-4 text-left transition-colors"
								>
									<SourceTypeIcon type={adapter.data_type} svg={adapter.icon_svg} size="lg" />
									<div className="min-w-0">
										<div className="text-ink truncate text-sm font-medium">{adapter.name}</div>
										<div className="text-ink-faint text-xs">v{adapter.version}</div>
									</div>
								</button>
							))}
						</div>
					</div>
				)}
			</div>

			{openCard && (
				<Modal onClose={close}>
					<AdapterDetail
						card={openCard}
						isolation={listing.data?.isolation ?? 'none'}
						onAddSource={() => {
							close();
							setConfigureId(openCard.id);
						}}
					/>
				</Modal>
			)}

			{configure && (
				<Modal onClose={() => setConfigureId(null)}>
					<div className="mb-4 flex items-center gap-3">
						<SourceTypeIcon type={configure.data_type} svg={configure.icon_svg} size="sm" />
						<h3 className="text-ink text-sm font-semibold">{configure.name}</h3>
					</div>
					<ConfigureForm
						adapterId={configure.id}
						adapterName={configure.name}
						onDone={() => setConfigureId(null)}
					/>
				</Modal>
			)}
		</>
	);
}

function TabButton({
	active,
	onClick,
	children
}: {
	active: boolean;
	onClick: () => void;
	children: React.ReactNode;
}) {
	return (
		<button
			onClick={onClick}
			className={`flex items-center rounded-md px-3 py-1 text-xs font-medium transition-colors ${
				active ? 'bg-app-selected text-ink' : 'text-ink-dull hover:text-ink'
			}`}
		>
			{children}
		</button>
	);
}

function Modal({onClose, children}: {onClose: () => void; children: React.ReactNode}) {
	useEffect(() => {
		const onKey = (event: KeyboardEvent) => event.key === 'Escape' && onClose();
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	}, [onClose]);
	return (
		<div
			className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4 backdrop-blur-sm"
			onClick={onClose}
		>
			<div
				role="dialog"
				aria-modal="true"
				className="border-app-line bg-app-box max-h-full w-full max-w-lg overflow-y-auto rounded-xl border p-6 shadow-2xl"
				onClick={(event) => event.stopPropagation()}
			>
				{children}
			</div>
		</div>
	);
}

function TrustBadge({trust}: {trust: string}) {
	const {t} = useTranslation('navigation');
	if (trust === 'community') return null;
	return (
		<span
			title={t(`store.trust.${trust}Hint`)}
			className="text-accent inline-flex shrink-0 items-center gap-0.5 text-[10px] font-medium"
		>
			<SealCheck size={12} weight="fill" />
			{t(`store.trust.${trust}`)}
		</span>
	);
}

function StateBadge({card}: {card: StoreCard}) {
	const {t} = useTranslation('navigation');
	if (card.revoked_reason) {
		return <Pill tone="red">{t('store.revoked')}</Pill>;
	}
	if (card.update_available) {
		return <Pill tone="blue">{t('sources.updateAvailable')}</Pill>;
	}
	if (card.installed_version) {
		return <Pill tone="green">{t('store.installed')}</Pill>;
	}
	if (!card.supported_here) {
		return <Pill tone="gray">{t('store.unsupported')}</Pill>;
	}
	return null;
}

function Pill({tone, children}: {tone: 'red' | 'blue' | 'green' | 'gray'; children: React.ReactNode}) {
	const tones = {
		red: 'bg-red-500/15 text-red-400 ring-red-500/30',
		blue: 'bg-blue-500/15 text-blue-400 ring-blue-500/30',
		green: 'bg-green-500/15 text-green-400 ring-green-500/30',
		gray: 'bg-app-line/60 text-ink-faint ring-app-line'
	};
	return (
		<span className={`shrink-0 rounded-full px-2 py-0.5 text-[10px] font-medium ring-1 ${tones[tone]}`}>
			{children}
		</span>
	);
}

function AdapterCard({card, onOpen}: {card: StoreCard; onOpen: () => void}) {
	const {t} = useTranslation('navigation');
	const text = useStoreText(card);
	return (
		<button
			onClick={onOpen}
			className="border-app-line bg-app-box hover:bg-app-hover flex h-full flex-col gap-3 rounded-xl border p-4 text-left transition-colors"
		>
			<div className="flex w-full items-start gap-3">
				<SourceTypeIcon type={card.id} svg={card.icon_svg} size="lg" />
				<div className="min-w-0 flex-1">
					<div className="flex items-center gap-2">
						<span className="text-ink truncate text-sm font-semibold">{card.name}</span>
						<TrustBadge trust={card.trust} />
					</div>
					<div className="text-ink-faint truncate text-[11px]">
						{t('sources.byAuthor', {author: card.author})}
					</div>
				</div>
				<StateBadge card={card} />
			</div>
			<p className="text-ink-dull line-clamp-2 text-xs leading-relaxed">{text.summary}</p>
		</button>
	);
}

function AdapterDetail({
	card,
	isolation,
	onAddSource
}: {
	card: StoreCard;
	isolation: string;
	onAddSource: () => void;
}) {
	const {t} = useTranslation('navigation');
	const text = useStoreText(card);
	const queryClient = useQueryClient();
	const install = useCoreMutation('adapters.store.install');
	const remove = useCoreMutation('adapters.store.remove');
	const [error, setError] = useState<string | null>(null);
	const busy = install.isPending || remove.isPending;

	const run = async (action: () => Promise<unknown>) => {
		setError(null);
		try {
			await action();
			await queryClient.invalidateQueries();
		} catch (e) {
			setError(String(e));
		}
	};

	const {read, network, secrets, tools} = card.permissions;
	const installable = card.supported_here && card.runtime_available;

	return (
		<div className="flex flex-col gap-4">
			<div className="flex items-start gap-3">
				<SourceTypeIcon type={card.id} svg={card.icon_svg} size="lg" />
				<div className="min-w-0 flex-1">
					<div className="flex items-center gap-2">
						<h3 className="text-ink text-base font-semibold">{card.name}</h3>
						<TrustBadge trust={card.trust} />
					</div>
					<div className="text-ink-faint text-xs">
						{t('store.meta', {
							version: card.installed_version ?? card.version,
							author: card.author,
							license: card.license
						})}
					</div>
				</div>
				<StateBadge card={card} />
			</div>

			<p className="text-ink-dull whitespace-pre-line text-sm leading-relaxed">
				{text.description || text.summary}
			</p>

			<div>
				<h4 className="text-ink mb-2 text-xs font-semibold">{t('store.permissions')}</h4>
				<ul className="text-ink-dull flex flex-col gap-1.5 text-xs">
					<Permission icon={HardDrives} label={t('store.permRead')} values={read.map(prettyPath)} />
					<Permission
						icon={Globe}
						label={t('store.permNetwork')}
						values={network}
						empty={t('store.noNetwork')}
					/>
					<Permission icon={Key} label={t('store.permSecrets')} values={secrets} />
					<Permission icon={Terminal} label={t('store.permTools')} values={tools} />
				</ul>
				<IsolationNote isolation={isolation} />
				{card.update_needs_consent && (
					<p className="mt-2 text-xs text-amber-400">{t('store.newPermissions')}</p>
				)}
			</div>

			{card.revoked_reason && (
				<Notice>{t('store.revokedReason', {reason: card.revoked_reason})}</Notice>
			)}
			{!card.supported_here && (
				<Notice>{t('store.unsupportedHint', {platforms: card.platforms.join(', ')})}</Notice>
			)}
			{card.supported_here && !card.runtime_available && <Notice>{t('store.needsPython')}</Notice>}

			{error && (
				<div className="rounded-md border border-red-400/20 p-2 text-xs text-red-400">{error}</div>
			)}

			<div className="flex flex-wrap items-center gap-2">
				{!card.installed_version && (
					<PrimaryButton
						disabled={busy || !installable}
						onClick={() =>
							void run(async () => {
								await install.mutateAsync({id: card.id, accept_permissions: true});
							})
						}
					>
						{install.isPending ? t('store.installing') : t('store.install')}
					</PrimaryButton>
				)}
				{card.installed_version && card.update_available && (
					<PrimaryButton
						disabled={busy || !installable}
						onClick={() =>
							void run(() => install.mutateAsync({id: card.id, accept_permissions: true}))
						}
					>
						{install.isPending
							? t('store.installing')
							: t('store.updateTo', {version: card.version})}
					</PrimaryButton>
				)}
				{card.installed_version && !card.revoked_reason && (
					<PrimaryButton disabled={busy} onClick={onAddSource}>
						{t('sources.add')}
					</PrimaryButton>
				)}
				{card.installed_version && (
					<button
						disabled={busy}
						onClick={() => void run(() => remove.mutateAsync({id: card.id}))}
						className="rounded-lg px-3.5 py-1.5 text-sm font-medium text-red-400 transition-colors hover:bg-red-500/10 disabled:opacity-40"
					>
						{remove.isPending ? t('store.removing') : t('store.remove')}
					</button>
				)}
				{card.homepage && (
					<a
						href={card.homepage}
						target="_blank"
						rel="noreferrer"
						className="text-ink-faint hover:text-ink ml-auto text-xs underline"
					>
						{t('store.homepage')}
					</a>
				)}
			</div>
			{!card.installed_version && installable && (
				<p className="text-ink-faint -mt-2 text-[11px]">{t('store.consent')}</p>
			)}
		</div>
	);
}

/** `{config.vault_path}` reads as "the folder you choose". */
function prettyPath(path: string) {
	return path.startsWith('{config.') ? `‹${path.slice(8, -1)}›` : path;
}

function Permission({
	icon: Icon,
	label,
	values,
	empty
}: {
	icon: typeof Globe;
	label: string;
	values: string[];
	empty?: string;
}) {
	if (values.length === 0 && !empty) return null;
	return (
		<li className="flex items-start gap-2">
			<Icon size={14} className="text-ink-faint mt-0.5 shrink-0" />
			<div className="min-w-0">
				<span className="text-ink">{label}</span>
				{values.length > 0 ? (
					<ul className="mt-0.5 flex flex-col gap-0.5">
						{values.map((value) => (
							<li key={value} className="text-ink-dull break-all font-mono text-[11px]">
								{value}
							</li>
						))}
					</ul>
				) : (
					<span className="text-ink-faint"> {empty}</span>
				)}
			</div>
		</li>
	);
}

/** Whether the access listed above is enforced on this system or only reviewed. */
function IsolationNote({isolation}: {isolation: string}) {
	const {t} = useTranslation('navigation');
	if (isolation === 'bubblewrap') {
		return (
			<p className="mt-2 flex items-start gap-1.5 text-xs text-green-400">
				<ShieldCheck size={14} className="mt-0.5 shrink-0" />
				{t('store.isolation.bubblewrap')}
			</p>
		);
	}
	const key = isolation === 'flatpak' || isolation === 'unavailable' ? isolation : 'none';
	return (
		<p className="mt-2 flex items-start gap-1.5 text-xs text-amber-400">
			<ShieldWarning size={14} className="mt-0.5 shrink-0" />
			{t(`store.isolation.${key}`)}
		</p>
	);
}

function Notice({children}: {children: React.ReactNode}) {
	return (
		<div className="flex items-start gap-2 rounded-md border border-amber-400/20 p-2 text-xs text-amber-400">
			<Warning size={14} className="mt-0.5 shrink-0" />
			<span>{children}</span>
		</div>
	);
}

function PrimaryButton({
	disabled,
	onClick,
	children
}: {
	disabled?: boolean;
	onClick: () => void;
	children: React.ReactNode;
}) {
	return (
		<button
			disabled={disabled}
			onClick={onClick}
			className="bg-accent hover:bg-accent-deep rounded-lg px-3.5 py-1.5 text-sm font-medium text-white transition-colors disabled:opacity-40"
		>
			{children}
		</button>
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
	const {t, i18n} = useTranslation('navigation');
	const navigate = useNavigate();
	const platform = usePlatform();
	const queryClient = useQueryClient();
	const language = (i18n.resolvedLanguage ?? i18n.language ?? 'en').split('-')[0];
	const label = (field: {name: string; translations?: Record<string, {name: string | null} | undefined>}) =>
		field.translations?.[language]?.name ?? field.name;
	const help = (field: {
		description: string;
		translations?: Record<string, {description: string | null} | undefined>;
	}) => field.translations?.[language]?.description ?? field.description;

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

	const [name, setName] = useState(adapterName);
	const [values, setValues] = useState<Record<string, string>>({});
	const [error, setError] = useState<string | null>(null);
	const [done, setDone] = useState(false);

	// Pre-fill what the adapter found on this machine.
	useEffect(() => {
		if (!fields) return;
		setValues((previous) => {
			const next = {...previous};
			for (const field of fields) {
				if (next[field.key] !== undefined) continue;
				if (field.suggestions.length > 0) {
					next[field.key] = field.suggestions[0];
				} else if (field.field_type === 'boolean') {
					next[field.key] = field.default === 'false' ? 'false' : 'true';
				}
			}
			return next;
		});
	}, [fields]);

	const handleFieldChange = useCallback((key: string, value: string) => {
		setValues((prev) => ({...prev, [key]: value}));
	}, []);

	const browse = async (key: string, title: string) => {
		const picked = await platform.openDirectoryPickerDialog?.({
			title,
			defaultPath: values[key] || undefined
		});
		const path = Array.isArray(picked) ? picked[0] : picked;
		if (path) handleFieldChange(key, path);
	};

	const handleSubmit = async () => {
		setError(null);
		if (!name.trim()) {
			setError(t('sources.nameRequired'));
			return;
		}

		try {
			const config: Record<string, unknown> = {};
			for (const field of fields ?? []) {
				const val = values[field.key]?.trim() ?? '';
				if (field.required && !val) {
					setError(t('sources.fieldRequired', {name: label(field)}));
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
			await syncSource.mutateAsync({source_id: source.id});
			// The sidebar and the sources page cached the list before this source existed.
			await queryClient.invalidateQueries();
			setDone(true);
		} catch (e) {
			setError(String(e));
		}
	};

	if (isLoading) {
		return <div className="text-ink-faint py-8 text-center text-sm">{t('store.detecting')}</div>;
	}

	if (queryError) {
		return (
			<div className="rounded-md border border-red-400/20 p-3 text-xs text-red-400">
				{t('sources.configLoadFailed', {error: String(queryError)})}
			</div>
		);
	}

	if (done) {
		return (
			<div className="flex flex-col items-center py-6">
				<CheckCircle size={28} className="text-green-400" weight="fill" />
				<p className="text-ink mt-2 text-sm">{t('sources.added')}</p>
				<p className="text-ink-faint mt-1 text-center text-xs">{t('sources.created')}</p>
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
				<label className="text-ink-dull mb-1 block text-xs font-medium">{t('sources.name')}</label>
				<input
					value={name}
					onChange={(e) => setName(e.target.value)}
					placeholder={t('sources.nameExample', {name: adapterName})}
					className="border-app-line bg-app-input text-ink focus:ring-accent w-full rounded-md border px-3 py-2 text-sm focus:outline-none focus:ring-1"
				/>
			</div>

			{fields?.map((field) =>
				field.field_type === 'boolean' ? (
					<label key={field.key} className="flex items-start gap-2">
						<input
							type="checkbox"
							checked={values[field.key] !== 'false'}
							onChange={(e) => handleFieldChange(field.key, String(e.target.checked))}
							className="accent-accent mt-0.5"
						/>
						<span>
							<span className="text-ink-dull block text-xs font-medium">{label(field)}</span>
							{help(field) && (
								<span className="text-ink-faint block text-[11px] leading-relaxed">
									{help(field)}
								</span>
							)}
						</span>
					</label>
				) : (
				<div key={field.key}>
					<label className="text-ink-dull mb-1 block text-xs font-medium">
						{label(field)}
						{field.required && <span className="ml-1 text-red-400">*</span>}
					</label>
					{help(field) && (
						<p className="text-ink-faint mb-1 text-[11px] leading-relaxed">{help(field)}</p>
					)}
					<div className="flex gap-2">
						<input
							type={field.secret ? 'password' : 'text'}
							value={values[field.key] ?? ''}
							list={field.suggestions.length > 1 ? `suggest-${field.key}` : undefined}
							onChange={(e) => handleFieldChange(field.key, e.target.value)}
							placeholder={
								field.default ? t('sources.defaultValue', {value: field.default}) : undefined
							}
							className="border-app-line bg-app-input text-ink focus:ring-accent w-full min-w-0 rounded-md border px-3 py-2 text-sm focus:outline-none focus:ring-1"
						/>
						{field.path_type === 'folder' && platform.openDirectoryPickerDialog && (
							<button
								type="button"
								title={t('store.browse')}
								onClick={() => void browse(field.key, label(field))}
								className="border-app-line text-ink-dull hover:text-ink hover:bg-app-hover shrink-0 rounded-md border px-2.5"
							>
								<FolderOpen size={16} />
							</button>
						)}
					</div>
					{field.suggestions.length > 1 && (
						<datalist id={`suggest-${field.key}`}>
							{field.suggestions.map((value) => (
								<option key={value} value={value} />
							))}
						</datalist>
					)}
					{field.suggestions.length > 0 && (
						<p className="text-ink-faint mt-1 text-[11px]">
							{t('store.detected', {count: field.suggestions.length})}
						</p>
					)}
				</div>
				)
			)}

			{error && (
				<div className="rounded-md border border-red-400/20 p-2 text-xs text-red-400">{error}</div>
			)}

			<div className="mt-1 flex gap-2">
				<button
					onClick={handleSubmit}
					disabled={createSource.isPending || syncSource.isPending}
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
