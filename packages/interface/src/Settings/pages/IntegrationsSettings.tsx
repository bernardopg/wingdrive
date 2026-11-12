import { useEffect, useId, useRef, useState } from "react";
import clsx from "clsx";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "@wingdrive/primitives";
import {
	WIRE_METHODS,
	type CloudSyncPairInfo,
	type GoogleService,
	type IntegrationAccountInfo,
	type PhotosPickerStatus,
	type SyncMode,
	type WingPath,
} from "@wingdrive/ts-client";
import { usePlatform } from "../../contexts/PlatformContext";
import { useLibraryMutation, useLibraryQuery, useWingDriveClient } from "../../contexts/WingDriveContext";
import { INTEGRATION_QUERY_KEYS, useGoogleConnect } from "../../hooks/useGoogleConnect";
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

const primaryButton = `${buttonClass} bg-accent text-white hover:bg-accent-deep`;
const secondaryButton = `${buttonClass} bg-app-button text-ink hover:bg-app-hover`;
const dangerButton = `${buttonClass} bg-status-error/10 text-status-error hover:bg-status-error/20`;

/** Minutes offered for automatic sync; 0 means manual only. */
const SYNC_INTERVALS = [0, 15, 60, 360, 1440];

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

	if (!libraryId || accounts.isLoading) {
		return <LibraryGate title={t("sections.integrations")} libraryId={libraryId} isLoading={accounts.isLoading} />;
	}

	const list = accounts.data ?? [];
	return (
		<div className="space-y-6">
			<SettingsHeader title={t("sections.integrations")} description={t("integrations.description")} />
			<GoogleAccountsSection accounts={list} />
			<CloudSyncSection accounts={list} />
			<CloudEditSection />
			<GoogleClientSection />
		</div>
	);
}

function GoogleAccountsSection({ accounts }: { accounts: IntegrationAccountInfo[] }) {
	const { t } = useTranslation();
	const clientInfo = useLibraryQuery({ type: "integrations.google.client", input: {} });
	const [drive, setDrive] = useState(true);
	const [photos, setPhotos] = useState(true);
	const { state, start, cancel, reset, isStarting } = useGoogleConnect((account) =>
		toast.success(t("integrations.google.connected", { email: account.email }))
	);
	const configured = clientInfo.data?.configured ?? false;
	const services: GoogleService[] = [...(drive ? (["drive"] as const) : []), ...(photos ? (["photos"] as const) : [])];

	return (
		<SettingsSection title={t("integrations.google.title")} description={t("integrations.google.description")}>
			{!configured && !clientInfo.isLoading && (
				<div className="px-4 py-3">
					<Notice tone="warning">{t("integrations.google.notConfigured")}</Notice>
				</div>
			)}
			{accounts.map((account) => (
				<AccountRow key={account.id} account={account} />
			))}
			{accounts.length === 0 && (
				<p className="px-4 py-3 text-sm text-ink-dull">{t("integrations.google.noAccounts")}</p>
			)}
			<div className="space-y-3 px-4 py-3">
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
				{state.phase === "waiting" ? (
					<div className="flex flex-wrap items-center gap-2">
						<span className="text-sm text-ink-dull">{t("integrations.google.waiting")}</span>
						<a className="text-sm text-accent underline" href={state.authUrl} target="_blank" rel="noreferrer">
							{t("integrations.google.openAgain")}
						</a>
						<button type="button" className={secondaryButton} onClick={() => void cancel()}>
							{t("cancel")}
						</button>
					</div>
				) : (
					<button
						type="button"
						className={primaryButton}
						disabled={!configured || isStarting || services.length === 0}
						onClick={() => void start(services)}
					>
						{t("integrations.google.connect")}
					</button>
				)}
				{state.phase === "error" && (
					<Notice tone="warning">
						<span>{t("integrations.google.failed", { error: state.message })}</span>{" "}
						<button type="button" className="underline" onClick={reset}>
							{t("integrations.dismiss")}
						</button>
					</Notice>
				)}
			</div>
		</SettingsSection>
	);
}

function AccountRow({ account }: { account: IntegrationAccountInfo }) {
	const { t } = useTranslation();
	const queryClient = useQueryClient();
	const mount = useLibraryMutation("integrations.google.drive.mount");
	const remove = useLibraryMutation("integrations.accounts.remove");
	const reconnect = useGoogleConnect(() => toast.success(t("integrations.google.reconnected")));
	const [confirming, setConfirming] = useState(false);
	const hasDrive = account.services.includes("drive");
	const hasPhotos = account.services.includes("photos");
	const refresh = () => {
		for (const queryKey of INTEGRATION_QUERY_KEYS) void queryClient.invalidateQueries({ queryKey: [...queryKey] });
	};

	const toggleDrive = async (mounted: boolean) => {
		try {
			await mount.mutateAsync({ account_id: account.id, mounted });
			refresh();
		} catch (error) {
			toast.error(t("integrations.google.driveFailed", { error: errorMessage(error) }));
		}
	};

	const disconnect = async () => {
		try {
			await remove.mutateAsync({ account_id: account.id, revoke: true });
			toast.success(t("integrations.google.disconnected", { email: account.email }));
			refresh();
		} catch (error) {
			toast.error(t("integrations.google.disconnectFailed", { error: errorMessage(error) }));
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
					<p className="truncate text-sm font-medium text-ink">{account.name ?? account.email}</p>
					<p className="truncate text-xs text-ink-dull">{account.email}</p>
				</div>
				<div className="flex flex-wrap justify-end gap-1">
					{account.services.map((service) => (
						<span key={service} className="rounded bg-app-selected px-2 py-0.5 text-xs text-ink-dull">
							{t(service === "drive" ? "integrations.google.serviceDrive" : "integrations.google.servicePhotos")}
						</span>
					))}
				</div>
			</div>

			{hasDrive && (
				<div className="flex items-center justify-between gap-3">
					<div className="min-w-0">
						<p className="text-sm text-ink">{t("integrations.google.driveMounted")}</p>
						<p className="text-xs text-ink-dull">
							{account.drive ? `gdrive://${account.drive.identifier}/` : t("integrations.google.driveHint")}
						</p>
					</div>
					<Switch
						checked={!!account.drive?.mounted}
						disabled={mount.isPending}
						onChange={(value) => void toggleDrive(value)}
					/>
				</div>
			)}

			{hasPhotos && <PhotosImport account={account} />}

			<div className="flex flex-wrap gap-2">
				{(!hasDrive || !hasPhotos) && (
					<button
						type="button"
						className={secondaryButton}
						disabled={reconnect.state.phase === "waiting"}
						onClick={() => void reconnect.start(["drive", "photos"], account.email)}
					>
						{reconnect.state.phase === "waiting" ? t("integrations.google.waiting") : t("integrations.google.grantMore")}
					</button>
				)}
				{confirming ? (
					<>
						<span className="self-center text-xs text-ink-dull">{t("integrations.google.confirmDisconnect")}</span>
						<button type="button" className={dangerButton} disabled={remove.isPending} onClick={() => void disconnect()}>
							{t("integrations.google.disconnect")}
						</button>
						<button type="button" className={secondaryButton} onClick={() => setConfirming(false)}>
							{t("cancel")}
						</button>
					</>
				) : (
					<button type="button" className={dangerButton} onClick={() => setConfirming(true)}>
						{t("integrations.google.disconnect")}
					</button>
				)}
			</div>
			{reconnect.state.phase === "error" && (
				<Notice tone="warning">{t("integrations.google.failed", { error: reconnect.state.message })}</Notice>
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
	const [account, setAccount] = useState(drives[0]?.drive?.identifier ?? "");
	const [remote, setRemote] = useState("WingDrive");
	const [mode, setMode] = useState<SyncMode>("two_way");
	const [propagate, setPropagate] = useState(false);
	const [interval, setSyncInterval] = useState(60);
	const ids = { name: useId(), local: useId(), account: useId(), remote: useId(), mode: useId(), interval: useId() };

	const submit = async () => {
		try {
			await create.mutateAsync({
				name: name.trim() || remote.trim() || "WingDrive",
				local_path: local.trim(),
				remote: { Cloud: { service: "gdrive", identifier: account, path: remote.trim().replace(/^\/+|\/+$/g, "") } },
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
						<option key={d.id} value={d.drive?.identifier}>
							{d.email}
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
				<button type="button" className={primaryButton} disabled={!local.trim() || !account || create.isPending} onClick={() => void submit()}>
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

/** The Google OAuth client: the build's or the user's own "Desktop app" client. */
function GoogleClientSection() {
	const { t } = useTranslation();
	const info = useLibraryQuery({ type: "integrations.google.client", input: {} });
	const save = useLibraryMutation("integrations.google.set_client");
	const [clientId, setClientId] = useState("");
	const [secret, setSecret] = useState("");
	const ids = { id: useId(), secret: useId() };
	const source = info.data?.source;

	const submit = async (clear: boolean) => {
		try {
			await save.mutateAsync(
				clear ? { client_id: null, client_secret: null } : { client_id: clientId.trim(), client_secret: secret.trim() }
			);
			setClientId("");
			setSecret("");
			await info.refetch();
			toast.success(t("integrations.client.saved"));
		} catch (error) {
			toast.error(t("integrations.client.failed", { error: errorMessage(error) }));
		}
	};

	return (
		<SettingsSection title={t("integrations.client.title")} description={t("integrations.client.description")}>
			<SettingsRow
				label={t("integrations.client.current")}
				hint={
					info.data?.configured
						? t(`integrations.client.source.${source ?? "bundled"}`, { id: info.data.client_id })
						: t("integrations.client.none")
				}
			>
				{source === "library" && (
					<button type="button" className={secondaryButton} disabled={save.isPending} onClick={() => void submit(true)}>
						{t("integrations.client.clear")}
					</button>
				)}
			</SettingsRow>
			<div className="space-y-2 px-4 py-3 text-xs text-ink-dull">
				<p>{t("integrations.client.steps")}</p>
				<ol className="list-decimal space-y-1 pl-5">
					<li>{t("integrations.client.step1")}</li>
					<li>{t("integrations.client.step2")}</li>
					<li>{t("integrations.client.step3")}</li>
					<li>{t("integrations.client.step4")}</li>
				</ol>
			</div>
			<SettingsRow label={t("integrations.client.clientId")} controlId={ids.id} stacked>
				<input
					id={ids.id}
					className={inputClass}
					value={clientId}
					placeholder="1234-abc.apps.googleusercontent.com"
					autoComplete="off"
					onChange={(e) => setClientId(e.target.value)}
				/>
			</SettingsRow>
			<SettingsRow label={t("integrations.client.clientSecret")} controlId={ids.secret} stacked>
				<input
					id={ids.secret}
					type="password"
					className={inputClass}
					value={secret}
					autoComplete="off"
					onChange={(e) => setSecret(e.target.value)}
				/>
			</SettingsRow>
			<div className="px-4 py-3">
				<button
					type="button"
					className={primaryButton}
					disabled={!clientId.trim() || !secret.trim() || save.isPending}
					onClick={() => void submit(false)}
				>
					{t("integrations.client.save")}
				</button>
			</div>
		</SettingsSection>
	);
}
