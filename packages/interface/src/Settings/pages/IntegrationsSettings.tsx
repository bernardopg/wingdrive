import { useEffect, useId, useRef, useState } from "react";
import clsx from "clsx";
import { useQueryClient } from "@tanstack/react-query";
import { CheckCircle, MinusCircle } from "@phosphor-icons/react";
import { toast } from "@wingdrive/primitives";
import {
	WIRE_METHODS,
	type CloudSyncPairInfo,
	type ConnectionMode,
	type GoogleService,
	type IntegrationAccountInfo,
	type IntegrationProvider,
	type OAuthClientsInfo,
	type PhotosPickerStatus,
	type SyncMode,
	type WingPath,
} from "@wingdrive/ts-client";
import { usePlatform } from "../../contexts/PlatformContext";
import { useLibraryMutation, useLibraryQuery, useWingDriveClient } from "../../contexts/WingDriveContext";
import { INTEGRATION_QUERY_KEYS, useIntegrationConnect } from "../../hooks/useIntegrationConnect";
import { useTranslation } from "../../i18n";
import {
	LibraryGate,
	Notice,
	SettingsHeader,
	SettingsRow,
	SettingsSection,
	Switch,
	SwitchRow,
	buttonClass,
	errorMessage,
	inputClass,
} from "../components";
import { ProviderLogo } from "./integrations/ProviderLogo";

const primaryButton = `${buttonClass} bg-accent text-white hover:bg-accent-deep`;
const secondaryButton = `${buttonClass} bg-app-button text-ink hover:bg-app-hover`;
const dangerButton = `${buttonClass} bg-status-error/10 text-status-error hover:bg-status-error/20`;

/** Minutes offered for automatic sync; 0 means manual only. */
const SYNC_INTERVALS = [0, 15, 60, 360, 1440];

/** One card of the catalog: a service and the way it connects. */
interface CatalogEntry {
	provider: IntegrationProvider;
	mode: ConnectionMode;
}

const CATALOG: CatalogEntry[] = [
	{ provider: "google", mode: "app" },
	{ provider: "google", mode: "custom" },
	{ provider: "microsoft", mode: "app" },
	{ provider: "microsoft", mode: "custom" },
];

/** i18n prefix of a catalog entry, such as `integrations.modes.google.app`. */
function modeKey(provider: IntegrationProvider, mode: ConnectionMode) {
	return `integrations.modes.${provider}.${mode}`;
}

function entryId(entry: CatalogEntry) {
	return `${entry.provider}-${entry.mode}`;
}

/** URI scheme of the drive an account mounts. */
function driveScheme(provider: IntegrationProvider) {
	return provider === "microsoft" ? "onedrive" : "gdrive";
}

function localPath(path: string): WingPath {
	return { Physical: { device_slug: "local", path } };
}

function cloudLabel(path: WingPath): string {
	if ("Cloud" in path) {
		const { identifier, path: inner } = path.Cloud;
		return inner ? `${identifier}/${inner}` : identifier;
	}
	if ("Physical" in path) return path.Physical.path;
	return "";
}

/** Asks the platform for a folder; null when the user cancels or no picker exists. */
async function pickFolder(
	platform: ReturnType<typeof usePlatform>,
	title: string
): Promise<string | null> {
	const picked = await platform.openDirectoryPickerDialog?.({ title });
	const path = Array.isArray(picked) ? picked[0] : picked;
	return path ?? null;
}

export function IntegrationsSettings() {
	const { t } = useTranslation();
	const client = useWingDriveClient();
	const libraryId = client.getCurrentLibraryId();
	const accounts = useLibraryQuery(
		{ type: "integrations.accounts.list", input: {} },
		{ enabled: !!libraryId }
	);
	const googleClients = useLibraryQuery({ type: "integrations.google.client", input: {} }, { enabled: !!libraryId });
	const microsoftClients = useLibraryQuery(
		{ type: "integrations.microsoft.client", input: {} },
		{ enabled: !!libraryId }
	);
	const [selected, setSelected] = useState<string | null>(null);

	if (!libraryId || accounts.isLoading) {
		return <LibraryGate title={t("sections.integrations")} libraryId={libraryId} isLoading={accounts.isLoading} />;
	}

	const list = accounts.data ?? [];
	const clients: Record<IntegrationProvider, OAuthClientsInfo | undefined> = {
		google: googleClients.data,
		microsoft: microsoftClients.data,
	};
	const current = CATALOG.find((entry) => entryId(entry) === selected);

	return (
		<div className="space-y-6">
			<SettingsHeader title={t("sections.integrations")} description={t("integrations.description")} />
			<IntegrationCatalog
				accounts={list}
				clients={clients}
				selected={selected}
				onSelect={(id) => setSelected((previous) => (previous === id ? null : id))}
			/>
			{current && (
				<ConnectPanel
					key={entryId(current)}
					entry={current}
					clients={clients[current.provider]}
					onSaved={() => void (current.provider === "google" ? googleClients : microsoftClients).refetch()}
				/>
			)}
			<ConnectedAccounts accounts={list} />
			<CloudSyncSection accounts={list} />
			<CloudEditSection />
		</div>
	);
}

/** Status of a catalog entry, shown as a chip on its card. */
function entryStatus(
	entry: CatalogEntry,
	clients: OAuthClientsInfo | undefined,
	t: (key: string, options?: Record<string, unknown>) => string
): { label: string; ready: boolean } {
	if (entry.mode === "app") {
		return clients?.app
			? { label: t("integrations.catalog.ready"), ready: true }
			: { label: t("integrations.catalog.unavailable"), ready: false };
	}
	return clients?.custom_client_id
		? { label: t("integrations.catalog.clientSaved"), ready: true }
		: { label: t("integrations.catalog.needsSetup"), ready: false };
}

function IntegrationCatalog({
	accounts,
	clients,
	selected,
	onSelect,
}: {
	accounts: IntegrationAccountInfo[];
	clients: Record<IntegrationProvider, OAuthClientsInfo | undefined>;
	selected: string | null;
	onSelect: (id: string) => void;
}) {
	const { t } = useTranslation();
	return (
		<section className="space-y-3" aria-labelledby="integrations-catalog-title">
			<div className="px-1">
				<h3 id="integrations-catalog-title" className="text-xs font-semibold uppercase tracking-wide text-ink-faint">
					{t("integrations.catalog.title")}
				</h3>
				<p className="mt-0.5 text-xs text-ink-dull">{t("integrations.catalog.description")}</p>
			</div>
			<div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
				{CATALOG.map((entry) => {
					const id = entryId(entry);
					const key = modeKey(entry.provider, entry.mode);
					const status = entryStatus(entry, clients[entry.provider], t);
					const connected = accounts.filter((a) => a.provider === entry.provider && a.mode === entry.mode).length;
					const isSelected = selected === id;
					return (
						<button
							key={id}
							type="button"
							aria-pressed={isSelected}
							aria-controls="integration-connect-panel"
							onClick={() => onSelect(id)}
							className={clsx(
								"flex min-h-40 flex-col items-start gap-3 rounded-lg border p-4 text-left transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent",
								isSelected ? "border-accent bg-accent/10" : "border-app-line bg-app-box hover:bg-app-hover"
							)}
						>
							<div className="flex w-full items-start justify-between gap-3">
								<ProviderLogo provider={entry.provider} className="size-9 shrink-0 text-ink" />
								<div className="flex flex-wrap justify-end gap-1">
									{connected > 0 && (
										<span className="rounded-full bg-accent/15 px-2 py-0.5 text-xs text-accent">
											{t("integrations.catalog.accounts", { count: connected })}
										</span>
									)}
									<span
										className={clsx(
											"rounded-full px-2 py-0.5 text-xs",
											status.ready ? "bg-app-selected text-ink-dull" : "bg-status-warning/10 text-status-warning"
										)}
									>
										{status.label}
									</span>
								</div>
							</div>
							<div>
								<p className="text-base font-semibold text-ink">{t(`integrations.providers.${entry.provider}`)}</p>
								<p className="mt-0.5 text-xs font-medium uppercase tracking-wide text-ink-dull">{t(`${key}.modality`)}</p>
							</div>
							<p className="text-sm text-ink-dull">{t(`${key}.summary`)}</p>
						</button>
					);
				})}
			</div>
		</section>
	);
}

/** Lines of a list stored in the catalog, such as the pros of a mode. */
function useLines(key: string): string[] {
	const { t } = useTranslation();
	const value = t(key, { returnObjects: true }) as unknown;
	return Array.isArray(value) ? value.map(String) : [];
}

/** Explains the selected service and mode, and connects an account with it. */
export function ConnectPanel({
	entry,
	clients,
	onSaved,
}: {
	entry: CatalogEntry;
	clients: OAuthClientsInfo | undefined;
	onSaved: () => void;
}) {
	const { t } = useTranslation();
	const key = modeKey(entry.provider, entry.mode);
	const pros = useLines(`${key}.pros`);
	const cons = useLines(`${key}.cons`);
	const steps = useLines(`${key}.steps`);
	const [drive, setDrive] = useState(true);
	const [photos, setPhotos] = useState(true);
	const { state, start, cancel, reset, isStarting } = useIntegrationConnect((account) =>
		toast.success(t("integrations.accounts.connected", { email: account.email }))
	);
	const services: GoogleService[] = [...(drive ? (["drive"] as const) : []), ...(photos ? (["photos"] as const) : [])];
	const ready = entry.mode === "app" ? !!clients?.app : !!clients?.custom_client_id;
	const canConnect = ready && !isStarting && (entry.provider !== "google" || services.length > 0);
	const titleId = useId();

	const connect = () =>
		void start(
			entry.provider === "google"
				? { provider: "google", mode: entry.mode, services }
				: { provider: "microsoft", mode: entry.mode }
		);

	return (
		<section
			id="integration-connect-panel"
			aria-labelledby={titleId}
			className="space-y-5 rounded-lg border border-app-line bg-app-box p-4"
		>
			<header className="flex items-center gap-3">
				<ProviderLogo provider={entry.provider} className="size-8 shrink-0 text-ink" />
				<div className="min-w-0">
					<h3 id={titleId} className="text-base font-semibold text-ink">
						{t(`integrations.providers.${entry.provider}`)} · {t(`${key}.modality`)}
					</h3>
					<p className="text-sm text-ink-dull">{t(`${key}.intro`)}</p>
				</div>
			</header>

			<div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
				<div>
					<h4 className="mb-2 text-xs font-semibold uppercase tracking-wide text-ink-faint">
						{t("integrations.catalog.pros")}
					</h4>
					<ul className="space-y-1.5">
						{pros.map((line) => (
							<li key={line} className="flex gap-2 text-sm text-ink">
								<CheckCircle weight="fill" className="mt-0.5 size-4 shrink-0 text-status-success" aria-hidden />
								<span>{line}</span>
							</li>
						))}
					</ul>
				</div>
				<div>
					<h4 className="mb-2 text-xs font-semibold uppercase tracking-wide text-ink-faint">
						{t("integrations.catalog.cons")}
					</h4>
					<ul className="space-y-1.5">
						{cons.map((line) => (
							<li key={line} className="flex gap-2 text-sm text-ink">
								<MinusCircle weight="fill" className="mt-0.5 size-4 shrink-0 text-ink-faint" aria-hidden />
								<span>{line}</span>
							</li>
						))}
					</ul>
				</div>
			</div>

			<div>
				<h4 className="mb-2 text-xs font-semibold uppercase tracking-wide text-ink-faint">
					{t("integrations.catalog.howTo")}
				</h4>
				<ol className="list-decimal space-y-1.5 pl-5 text-sm text-ink">
					{steps.map((line) => (
						<li key={line}>{line}</li>
					))}
				</ol>
			</div>

			{entry.mode === "custom" &&
				(entry.provider === "google" ? (
					<GoogleClientForm clients={clients} onSaved={onSaved} />
				) : (
					<MicrosoftClientForm clients={clients} onSaved={onSaved} />
				))}

			{entry.mode === "app" && !clients?.app && <Notice tone="warning">{t(`${key}.unavailable`)}</Notice>}

			<div className="space-y-3 border-t border-app-line pt-4">
				{entry.provider === "google" && (
					<div className="flex flex-wrap gap-4 text-sm text-ink">
						<label className="flex items-center gap-2">
							<input type="checkbox" checked={drive} onChange={(e) => setDrive(e.target.checked)} />
							{t("integrations.google.serviceDrive")}
						</label>
						<label className="flex items-center gap-2">
							<input type="checkbox" checked={photos} onChange={(e) => setPhotos(e.target.checked)} />
							{t("integrations.google.servicePhotos")}
						</label>
					</div>
				)}
				{state.phase === "waiting" ? (
					<div className="flex flex-wrap items-center gap-2">
						<span className="text-sm text-ink-dull">{t("integrations.accounts.waiting")}</span>
						<a className="text-sm text-accent underline" href={state.authUrl} target="_blank" rel="noreferrer">
							{t("integrations.accounts.openAgain")}
						</a>
						<button type="button" className={secondaryButton} onClick={() => void cancel()}>
							{t("cancel")}
						</button>
					</div>
				) : (
					<button type="button" className={primaryButton} disabled={!canConnect} onClick={connect}>
						{t(`${key}.connect`)}
					</button>
				)}
				{state.phase === "error" && (
					<Notice tone="warning">
						<span>{t("integrations.accounts.failed", { error: state.message })}</span>{" "}
						<button type="button" className="underline" onClick={reset}>
							{t("integrations.dismiss")}
						</button>
					</Notice>
				)}
			</div>
		</section>
	);
}

/** Saves or clears the user's own "Desktop app" client from Google Cloud. */
function GoogleClientForm({ clients, onSaved }: { clients: OAuthClientsInfo | undefined; onSaved: () => void }) {
	const { t } = useTranslation();
	const save = useLibraryMutation("integrations.google.set_client");
	const [clientId, setClientId] = useState("");
	const [secret, setSecret] = useState("");
	const ids = { id: useId(), secret: useId() };

	const submit = async (clear: boolean) => {
		try {
			await save.mutateAsync(
				clear ? { client_id: null, client_secret: null } : { client_id: clientId.trim(), client_secret: secret.trim() }
			);
			setClientId("");
			setSecret("");
			onSaved();
			toast.success(t(clear ? "integrations.client.cleared" : "integrations.client.saved"));
		} catch (error) {
			toast.error(t("integrations.client.failed", { error: errorMessage(error) }));
		}
	};

	return (
		<div className="space-y-3 rounded-lg border border-app-line p-3">
			<SavedClient clientId={clients?.custom_client_id} disabled={save.isPending} onClear={() => void submit(true)} />
			<div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
				<div className="space-y-1">
					<label htmlFor={ids.id} className="block text-sm text-ink">
						{t("integrations.client.clientId")}
					</label>
					<input
						id={ids.id}
						className={inputClass}
						value={clientId}
						placeholder="1234-abc.apps.googleusercontent.com"
						autoComplete="off"
						onChange={(e) => setClientId(e.target.value)}
					/>
				</div>
				<div className="space-y-1">
					<label htmlFor={ids.secret} className="block text-sm text-ink">
						{t("integrations.client.clientSecret")}
					</label>
					<input
						id={ids.secret}
						type="password"
						className={inputClass}
						value={secret}
						autoComplete="off"
						onChange={(e) => setSecret(e.target.value)}
					/>
				</div>
			</div>
			<button
				type="button"
				className={secondaryButton}
				disabled={!clientId.trim() || !secret.trim() || save.isPending}
				onClick={() => void submit(false)}
			>
				{t("integrations.client.save")}
			</button>
		</div>
	);
}

/** Saves or clears the user's own public client app from Microsoft Entra. */
function MicrosoftClientForm({ clients, onSaved }: { clients: OAuthClientsInfo | undefined; onSaved: () => void }) {
	const { t } = useTranslation();
	const save = useLibraryMutation("integrations.microsoft.set_client");
	const [clientId, setClientId] = useState("");
	const inputId = useId();

	const submit = async (clear: boolean) => {
		try {
			await save.mutateAsync({ client_id: clear ? null : clientId.trim() });
			setClientId("");
			onSaved();
			toast.success(t(clear ? "integrations.client.cleared" : "integrations.client.saved"));
		} catch (error) {
			toast.error(t("integrations.client.failed", { error: errorMessage(error) }));
		}
	};

	return (
		<div className="space-y-3 rounded-lg border border-app-line p-3">
			<SavedClient clientId={clients?.custom_client_id} disabled={save.isPending} onClear={() => void submit(true)} />
			<div className="space-y-1">
				<label htmlFor={inputId} className="block text-sm text-ink">
					{t("integrations.client.applicationId")}
				</label>
				<input
					id={inputId}
					className={inputClass}
					value={clientId}
					placeholder="00000000-0000-0000-0000-000000000000"
					autoComplete="off"
					onChange={(e) => setClientId(e.target.value)}
				/>
			</div>
			<button
				type="button"
				className={secondaryButton}
				disabled={!clientId.trim() || save.isPending}
				onClick={() => void submit(false)}
			>
				{t("integrations.client.save")}
			</button>
		</div>
	);
}

function SavedClient({
	clientId,
	disabled,
	onClear,
}: {
	clientId: string | null | undefined;
	disabled: boolean;
	onClear: () => void;
}) {
	const { t } = useTranslation();
	return (
		<div className="flex flex-wrap items-center justify-between gap-2">
			<p className="min-w-0 break-all text-sm text-ink-dull">
				{clientId ? t("integrations.client.current", { id: clientId }) : t("integrations.client.none")}
			</p>
			{clientId && (
				<button type="button" className={secondaryButton} disabled={disabled} onClick={onClear}>
					{t("integrations.client.clear")}
				</button>
			)}
		</div>
	);
}

function ConnectedAccounts({ accounts }: { accounts: IntegrationAccountInfo[] }) {
	const { t } = useTranslation();
	return (
		<SettingsSection title={t("integrations.accounts.title")} description={t("integrations.accounts.description")}>
			{accounts.map((account) => (
				<AccountRow key={`${account.provider}-${account.id}`} account={account} />
			))}
			{accounts.length === 0 && <p className="px-4 py-3 text-sm text-ink-dull">{t("integrations.accounts.none")}</p>}
		</SettingsSection>
	);
}

function AccountRow({ account }: { account: IntegrationAccountInfo }) {
	const { t } = useTranslation();
	const queryClient = useQueryClient();
	const mountGoogle = useLibraryMutation("integrations.google.drive.mount");
	const mountOneDrive = useLibraryMutation("integrations.onedrive.mount");
	const remove = useLibraryMutation("integrations.accounts.remove");
	const reconnect = useIntegrationConnect(() => toast.success(t("integrations.google.reconnected")));
	const [confirming, setConfirming] = useState(false);
	const microsoft = account.provider === "microsoft";
	const hasDrive = microsoft || account.services.includes("drive");
	const hasPhotos = account.services.includes("photos");
	const mounting = mountGoogle.isPending || mountOneDrive.isPending;
	const refresh = () => {
		for (const queryKey of INTEGRATION_QUERY_KEYS) void queryClient.invalidateQueries({ queryKey: [...queryKey] });
	};

	const toggleDrive = async (mounted: boolean) => {
		try {
			if (microsoft) await mountOneDrive.mutateAsync({ account_id: account.id, mounted });
			else await mountGoogle.mutateAsync({ account_id: account.id, mounted });
			refresh();
		} catch (error) {
			toast.error(t("integrations.accounts.driveFailed", { error: errorMessage(error) }));
		}
	};

	const disconnect = async () => {
		try {
			await remove.mutateAsync({ account_id: account.id, revoke: true });
			toast.success(t("integrations.accounts.disconnected", { email: account.email }));
			refresh();
		} catch (error) {
			toast.error(t("integrations.accounts.disconnectFailed", { error: errorMessage(error) }));
		} finally {
			setConfirming(false);
		}
	};

	return (
		<div className="space-y-3 px-4 py-3">
			<div className="flex items-center gap-3">
				{account.picture ? (
					<img src={account.picture} alt="" className="size-9 rounded-full" referrerPolicy="no-referrer" />
				) : (
					<div className="flex size-9 items-center justify-center rounded-full bg-accent/20 text-sm font-semibold text-accent">
						{(account.name ?? account.email).slice(0, 1).toUpperCase()}
					</div>
				)}
				<div className="min-w-0 flex-1">
					<p className="flex items-center gap-1.5 truncate text-sm font-medium text-ink">
						<ProviderLogo provider={account.provider} className="size-3.5 shrink-0 text-ink-dull" />
						<span className="truncate">{account.name ?? account.email}</span>
					</p>
					<p className="truncate text-xs text-ink-dull">{account.email}</p>
				</div>
				<div className="flex flex-wrap justify-end gap-1">
					<span className="rounded-full bg-app-selected px-2 py-0.5 text-xs text-ink-dull">
						{t(`${modeKey(account.provider, account.mode)}.modality`)}
					</span>
					{microsoft ? (
						<span className="rounded bg-app-selected px-2 py-0.5 text-xs text-ink-dull">OneDrive</span>
					) : (
						account.services.map((service) => (
							<span key={service} className="rounded bg-app-selected px-2 py-0.5 text-xs text-ink-dull">
								{t(service === "drive" ? "integrations.google.serviceDrive" : "integrations.google.servicePhotos")}
							</span>
						))
					)}
				</div>
			</div>

			{hasDrive && (
				<div className="flex items-center justify-between gap-3">
					<div className="min-w-0">
						<p className="text-sm text-ink">
							{t(microsoft ? "integrations.accounts.oneDriveMounted" : "integrations.google.driveMounted")}
						</p>
						<p className="text-xs text-ink-dull">
							{account.drive
								? `${driveScheme(account.provider)}://${account.drive.identifier}/`
								: t("integrations.accounts.driveHint")}
						</p>
					</div>
					<Switch checked={!!account.drive?.mounted} disabled={mounting} onChange={(value) => void toggleDrive(value)} />
				</div>
			)}

			{hasPhotos && <PhotosImport account={account} />}

			<div className="flex flex-wrap gap-2">
				{!microsoft && (!hasDrive || !hasPhotos) && (
					<button
						type="button"
						className={secondaryButton}
						disabled={reconnect.state.phase === "waiting"}
						onClick={() =>
							void reconnect.start({
								provider: "google",
								mode: account.mode,
								services: ["drive", "photos"],
								loginHint: account.email,
							})
						}
					>
						{reconnect.state.phase === "waiting" ? t("integrations.accounts.waiting") : t("integrations.google.grantMore")}
					</button>
				)}
				{confirming ? (
					<>
						<span className="self-center text-xs text-ink-dull">
							{t(microsoft ? "integrations.accounts.confirmDisconnectMicrosoft" : "integrations.accounts.confirmDisconnect")}
						</span>
						<button type="button" className={dangerButton} disabled={remove.isPending} onClick={() => void disconnect()}>
							{t("integrations.accounts.disconnect")}
						</button>
						<button type="button" className={secondaryButton} onClick={() => setConfirming(false)}>
							{t("cancel")}
						</button>
					</>
				) : (
					<button type="button" className={dangerButton} onClick={() => setConfirming(true)}>
						{t("integrations.accounts.disconnect")}
					</button>
				)}
			</div>
			{reconnect.state.phase === "error" && (
				<Notice tone="warning">{t("integrations.accounts.failed", { error: reconnect.state.message })}</Notice>
			)}
		</div>
	);
}

/** Opens the Google Photos picker and downloads the selection into a folder. */
function PhotosImport({ account }: { account: IntegrationAccountInfo }) {
	const { t } = useTranslation();
	const platform = usePlatform();
	const client = useWingDriveClient();
	const startPicker = useLibraryMutation("integrations.google.photos.picker.start");
	const importPicked = useLibraryMutation("integrations.google.photos.picker.import");
	const [destination, setDestination] = useState("");
	const [waiting, setWaiting] = useState(false);
	const active = useRef(true);
	const inputId = useId();
	useEffect(
		() => () => {
			active.current = false;
		},
		[]
	);

	const run = async () => {
		const folder = destination.trim() || (await pickFolder(platform, t("integrations.photos.destination")));
		if (!folder) return;
		setDestination(folder);
		try {
			const session = await startPicker.mutateAsync({ account_id: account.id });
			platform.openLink(session.picker_uri);
			setWaiting(true);
			// Google suggests the interval; the picker session itself expires after some minutes.
			for (;;) {
				await new Promise((resolve) => setTimeout(resolve, Math.max(1, session.poll_interval_seconds) * 1000));
				if (!active.current) return;
				const status = await client.execute<{ account_id: string; session_id: string }, PhotosPickerStatus>(
					WIRE_METHODS.libraryQueries["integrations.google.photos.picker.status"],
					{ account_id: account.id, session_id: session.session_id }
				);
				if (status.ready) break;
			}
			await importPicked.mutateAsync({
				account_id: account.id,
				session_id: session.session_id,
				destination: localPath(folder),
			});
			toast.success(t("integrations.photos.importStarted", { folder }));
		} catch (error) {
			toast.error(t("integrations.photos.importFailed", { error: errorMessage(error) }));
		} finally {
			setWaiting(false);
		}
	};

	return (
		<div className="space-y-2">
			<label htmlFor={inputId} className="block text-sm text-ink">
				{t("integrations.photos.import")}
			</label>
			<p className="text-xs text-ink-dull">{t("integrations.photos.importHint")}</p>
			<div className="flex gap-2">
				<input
					id={inputId}
					className={inputClass}
					value={destination}
					placeholder={t("integrations.photos.destinationPlaceholder")}
					onChange={(e) => setDestination(e.target.value)}
				/>
				<button type="button" className={`${secondaryButton} shrink-0`} disabled={waiting} onClick={() => void run()}>
					{waiting ? t("integrations.photos.waiting") : t("integrations.photos.choose")}
				</button>
			</div>
		</div>
	);
}

function pairStatus(pair: CloudSyncPairInfo, t: (key: string, options?: Record<string, unknown>) => string): string {
	if (pair.running) return t("integrations.sync.running");
	if (!pair.last_run_at) return t("integrations.sync.never");
	const when = new Date(pair.last_run_at).toLocaleString();
	const report = pair.last_report;
	if (pair.last_status === "failed") return t("integrations.sync.failedAt", { when, error: pair.last_error ?? "" });
	if (!report) return t("integrations.sync.ranAt", { when });
	return t("integrations.sync.summary", {
		when,
		uploaded: report.uploaded,
		downloaded: report.downloaded,
		conflicts: report.conflicts,
		errors: report.errors.length,
	});
}

function CloudSyncSection({ accounts }: { accounts: IntegrationAccountInfo[] }) {
	const { t } = useTranslation();
	const pairs = useLibraryQuery({ type: "cloud_sync.list", input: {} }, { refetchInterval: 5000 });
	const run = useLibraryMutation("cloud_sync.run");
	const update = useLibraryMutation("cloud_sync.update");
	const remove = useLibraryMutation("cloud_sync.delete");
	const [adding, setAdding] = useState(false);
	const drives = accounts.filter((a) => a.drive?.mounted);

	const act = async (action: () => Promise<unknown>) => {
		try {
			await action();
			await pairs.refetch();
		} catch (error) {
			toast.error(t("integrations.sync.actionFailed", { error: errorMessage(error) }));
		}
	};

	return (
		<SettingsSection title={t("integrations.sync.title")} description={t("integrations.sync.description")}>
			{(pairs.data ?? []).map((pair) => (
				<div key={pair.id} className="space-y-2 px-4 py-3">
					<div className="flex items-start justify-between gap-3">
						<div className="min-w-0">
							<p className="truncate text-sm font-medium text-ink">{pair.name}</p>
							<p className="truncate text-xs text-ink-dull">
								{pair.local_path} {pair.mode === "upload" ? "→" : pair.mode === "download" ? "←" : "⇄"} {cloudLabel(pair.remote)}
							</p>
							<p className={clsx("text-xs", pair.last_status === "failed" ? "text-status-error" : "text-ink-faint")}>
								{pairStatus(pair, t)}
							</p>
						</div>
						<Switch
							checked={pair.enabled}
							onChange={(enabled) => void act(() => update.mutateAsync({ id: pair.id, name: null, mode: null, propagate_deletes: null, interval_minutes: null, enabled }))}
						/>
					</div>
					<div className="flex gap-2">
						<button
							type="button"
							className={secondaryButton}
							disabled={pair.running || run.isPending}
							onClick={() => void act(() => run.mutateAsync({ id: pair.id }))}
						>
							{t("integrations.sync.runNow")}
						</button>
						<button
							type="button"
							className={dangerButton}
							disabled={pair.running}
							onClick={() => void act(() => remove.mutateAsync({ id: pair.id }))}
						>
							{t("integrations.sync.remove")}
						</button>
					</div>
				</div>
			))}
			{(pairs.data ?? []).length === 0 && <p className="px-4 py-3 text-sm text-ink-dull">{t("integrations.sync.empty")}</p>}
			<div className="px-4 py-3">
				{adding ? (
					<NewSyncPair drives={drives} onDone={() => { setAdding(false); void pairs.refetch(); }} />
				) : (
					<button type="button" className={secondaryButton} disabled={drives.length === 0} onClick={() => setAdding(true)}>
						{t("integrations.sync.add")}
					</button>
				)}
				{drives.length === 0 && <p className="mt-2 text-xs text-ink-dull">{t("integrations.sync.needsDrive")}</p>}
			</div>
		</SettingsSection>
	);
}

function NewSyncPair({ drives, onDone }: { drives: IntegrationAccountInfo[]; onDone: () => void }) {
	const { t } = useTranslation();
	const platform = usePlatform();
	const create = useLibraryMutation("cloud_sync.create");
	const [name, setName] = useState("");
	const [local, setLocal] = useState("");
	const [account, setAccount] = useState(drives[0] ? `${drives[0].provider}:${drives[0].id}` : "");
	const [remote, setRemote] = useState("WingDrive");
	const [mode, setMode] = useState<SyncMode>("two_way");
	const [propagate, setPropagate] = useState(false);
	const [interval, setSyncInterval] = useState(60);
	const ids = { name: useId(), local: useId(), account: useId(), remote: useId(), mode: useId(), interval: useId() };
	const target = drives.find((d) => `${d.provider}:${d.id}` === account);

	const submit = async () => {
		if (!target?.drive) return;
		try {
			await create.mutateAsync({
				name: name.trim() || remote.trim() || "WingDrive",
				local_path: local.trim(),
				remote: {
					Cloud: {
						service: target.provider === "microsoft" ? "onedrive" : "gdrive",
						identifier: target.drive.identifier,
						path: remote.trim().replace(/^\/+|\/+$/g, ""),
					},
				},
				mode,
				propagate_deletes: propagate,
				interval_minutes: interval > 0 ? interval : null,
				run_now: true,
			});
			toast.success(t("integrations.sync.created"));
			onDone();
		} catch (error) {
			toast.error(t("integrations.sync.actionFailed", { error: errorMessage(error) }));
		}
	};

	return (
		<div className="space-y-3">
			<SettingsRow label={t("integrations.sync.name")} controlId={ids.name} stacked>
				<input id={ids.name} className={inputClass} value={name} onChange={(e) => setName(e.target.value)} />
			</SettingsRow>
			<SettingsRow label={t("integrations.sync.localFolder")} controlId={ids.local} stacked>
				<div className="flex gap-2">
					<input id={ids.local} className={inputClass} value={local} placeholder="/home/…" onChange={(e) => setLocal(e.target.value)} />
					{platform.openDirectoryPickerDialog && (
						<button
							type="button"
							className={`${secondaryButton} shrink-0`}
							onClick={async () => {
								const folder = await pickFolder(platform, t("integrations.sync.localFolder"));
								if (folder) setLocal(folder);
							}}
						>
							{t("integrations.photos.choose")}
						</button>
					)}
				</div>
			</SettingsRow>
			<SettingsRow label={t("integrations.sync.account")} controlId={ids.account} stacked>
				<select id={ids.account} className={inputClass} value={account} onChange={(e) => setAccount(e.target.value)}>
					{drives.map((d) => (
						<option key={`${d.provider}:${d.id}`} value={`${d.provider}:${d.id}`}>
							{t(d.provider === "microsoft" ? "integrations.sync.accountOneDrive" : "integrations.sync.accountGoogle", {
								email: d.email,
							})}
						</option>
					))}
				</select>
			</SettingsRow>
			<SettingsRow label={t("integrations.sync.remoteFolder")} hint={t("integrations.sync.remoteFolderHint")} controlId={ids.remote} stacked>
				<input id={ids.remote} className={inputClass} value={remote} onChange={(e) => setRemote(e.target.value)} />
			</SettingsRow>
			<SettingsRow label={t("integrations.sync.mode")} controlId={ids.mode} stacked>
				<select id={ids.mode} className={inputClass} value={mode} onChange={(e) => setMode(e.target.value as SyncMode)}>
					<option value="two_way">{t("integrations.sync.modeTwoWay")}</option>
					<option value="upload">{t("integrations.sync.modeUpload")}</option>
					<option value="download">{t("integrations.sync.modeDownload")}</option>
				</select>
			</SettingsRow>
			<SettingsRow label={t("integrations.sync.interval")} controlId={ids.interval} stacked>
				<select id={ids.interval} className={inputClass} value={interval} onChange={(e) => setSyncInterval(Number(e.target.value))}>
					{SYNC_INTERVALS.map((minutes) => (
						<option key={minutes} value={minutes}>
							{minutes === 0 ? t("integrations.sync.manual") : t("integrations.sync.everyMinutes", { count: minutes })}
						</option>
					))}
				</select>
			</SettingsRow>
			<SwitchRow
				label={t("integrations.sync.propagateDeletes")}
				hint={t("integrations.sync.propagateDeletesHint")}
				checked={propagate}
				onChange={setPropagate}
			/>
			<div className="flex gap-2">
				<button type="button" className={primaryButton} disabled={!local.trim() || !target || create.isPending} onClick={() => void submit()}>
					{t("integrations.sync.create")}
				</button>
				<button type="button" className={secondaryButton} onClick={onDone}>
					{t("cancel")}
				</button>
			</div>
		</div>
	);
}

/** Cloud files open in local apps, with their upload state. */
function CloudEditSection() {
	const { t } = useTranslation();
	const sessions = useLibraryQuery({ type: "files.cloud_edit.list", input: {} }, { refetchInterval: 3000 });
	const close = useLibraryMutation("files.cloud_edit.close");
	const list = sessions.data ?? [];
	if (list.length === 0) return null;

	const stateLabel = (state: (typeof list)[number]["state"]) => {
		switch (state.state) {
			case "synced":
				return t("integrations.edit.synced");
			case "modified":
				return t("integrations.edit.modified");
			case "uploading":
				return t("integrations.edit.uploading");
			case "conflict":
				return t("integrations.edit.conflict", { name: state.copy_name });
			case "error":
				return t("integrations.edit.error", { error: state.message });
		}
	};

	return (
		<SettingsSection title={t("integrations.edit.title")} description={t("integrations.edit.description")}>
			{list.map((session) => (
				<div key={session.id} className="flex items-center justify-between gap-3 px-4 py-3">
					<div className="min-w-0">
						<p className="truncate text-sm text-ink">{cloudLabel(session.path)}</p>
						<p className="text-xs text-ink-dull">{stateLabel(session.state)}</p>
					</div>
					<button
						type="button"
						className={secondaryButton}
						onClick={async () => {
							try {
								await close.mutateAsync({ id: session.id });
								await sessions.refetch();
							} catch (error) {
								toast.error(t("integrations.edit.closeFailed", { error: errorMessage(error) }));
							}
						}}
					>
						{t("integrations.edit.finish")}
					</button>
				</div>
			))}
		</SettingsSection>
	);
}
