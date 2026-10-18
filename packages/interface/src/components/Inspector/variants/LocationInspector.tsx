import {
	Info,
	Gear,
	Briefcase,
	ClockCounterClockwise,
	HardDrive,
	DotsThree,
	Sparkle,
	Image,
	MagnifyingGlass,
	Trash,
	FunnelX,
	ToggleLeft,
	ToggleRight,
	X,
	Play,
	FilmStrip,
	VideoCamera,
	FolderOpen,
	ArrowsClockwise,
} from "@phosphor-icons/react";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { useQueryClient } from "@tanstack/react-query";
import { useLocation, useNavigate } from "react-router-dom";
import {
	InfoRow,
	Section,
	Divider,
	Tabs,
	TabContent,
} from "../Inspector";
import clsx from "clsx";
import type { Location, ScanState } from "@wingdrive/ts-client";
import { Button, Dialog, dialogManager, useDialog, CircleButton, type UseDialogProps } from "@wingdrive/primitives";
import { useLibraryMutation } from "../../../contexts/WingDriveContext";
import { useContextMenu } from "../../../hooks/useContextMenu";
import LocationIcon from "@wingdrive/assets/icons/Location.webp";
import { useTranslation } from "../../../i18n";
import { ThemedIcon } from "../../ThemedIcon";
import {uiLocale} from "../../../i18n";

interface LocationInspectorProps {
	location: Location;
}

export function LocationInspector({ location }: LocationInspectorProps) {
const { t } = useTranslation("inspector");
	const [activeTab, setActiveTab] = useState("overview");

	const tabs = [
{ id: "overview", label: t("inspector.location.overview"), icon: Info },
{ id: "indexing", label: t("inspector.location.indexing"), icon: Gear },
{ id: "jobs", label: t("inspector.location.jobs"), icon: Briefcase },
{ id: "activity", label: t("inspector.location.activity"), icon: ClockCounterClockwise },
{ id: "devices", label: t("inspector.location.devices"), icon: HardDrive },
{ id: "more", label: t("inspector.location.more"), icon: DotsThree },
	];

	return (
		<>
			{/* Tabs */}
			<Tabs tabs={tabs} activeTab={activeTab} onChange={setActiveTab} />

			{/* Tab Content */}
			<div className="flex-1 overflow-hidden flex flex-col mt-2.5">
				<TabContent id="overview" activeTab={activeTab}>
					<OverviewTab location={location} />
				</TabContent>

				<TabContent id="indexing" activeTab={activeTab}>
					<IndexingTab location={location} />
				</TabContent>

				<TabContent id="jobs" activeTab={activeTab}>
					<JobsTab location={location} />
				</TabContent>

				<TabContent id="activity" activeTab={activeTab}>
					<ActivityTab location={location} />
				</TabContent>

				<TabContent id="devices" activeTab={activeTab}>
					<DevicesTab location={location} />
				</TabContent>

				<TabContent id="more" activeTab={activeTab}>
					<MoreTab location={location} />
				</TabContent>
			</div>
		</>
	);
}

function OverviewTab({ location }: { location: Location }) {
const { t } = useTranslation("inspector");
	const rescanLocation = useLibraryMutation("locations.rescan");
	const routeLocation = useLocation();
	const navigate = useNavigate();
	const isOverview = routeLocation.pathname === '/';

	const reindexMenu = useContextMenu({
		items: [
			{
				icon: MagnifyingGlass,
label: t("inspector.location.quickReindex"),
				onClick: () => {
					rescanLocation.mutate({
						location_id: location.id,
						full_rescan: false,
					});
				},
			},
			{
				icon: Sparkle,
label: t("inspector.location.fullReindex"),
				onClick: () => {
					rescanLocation.mutate({
						location_id: location.id,
						full_rescan: true,
					});
				},
			},
		],
	});

	const formatBytes = (bytes: number | null | undefined) => {
		if (!bytes || bytes === 0) return "0 B";
		const k = 1024;
		const sizes = ["B", "KB", "MB", "GB", "TB"];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
	};

	const formatDate = (dateStr: string) => {
		const date = new Date(dateStr);
		return date.toLocaleDateString(uiLocale(), {
			month: "short",
			day: "numeric",
			year: "numeric",
			hour: "2-digit",
			minute: "2-digit",
		});
	};

	// Unit variants serialize as plain strings; only Scanning carries data
const formatScanState = (scanState: ScanState | null | undefined) => {
if (!scanState) return t("inspector.location.unknown");
		if (typeof scanState === "string") return t(`inspector.location.scanStates.${scanState}`, { defaultValue: scanState });
if ("Scanning" in scanState) return t("inspector.location.scanning", { progress: scanState.Scanning.progress });
return t("inspector.location.unknown");
	};

	return (
		<div className="no-scrollbar mask-fade-out flex flex-col space-y-5 overflow-x-hidden overflow-y-scroll pb-10">
			{/* Location icon */}
			<div className="flex justify-center h-48 items-center w-full px-4">
<ThemedIcon src={LocationIcon} className="size-24" alt={t("inspector.location.location")} />
			</div>

			{/* Location name */}
			<div className="px-2 text-center">
				<h4 className="text-sm font-semibold text-sidebar-ink truncate">
{location.name || t("inspector.location.unnamed")}
				</h4>
				<p className="text-xs text-sidebar-ink-dull mt-0.5">
{t("inspector.location.localStorage")}
				</p>
			</div>

			<Divider />

			{/* Action Buttons */}
			<div className="px-2 mb-5 flex gap-2">
				{isOverview && (
					<CircleButton
						icon={FolderOpen}
						onClick={() => {
							const encodedPath = encodeURIComponent(JSON.stringify(location.wing_path));
							navigate(`/explorer?path=${encodedPath}`);
						}}
						className="flex-1"
					>
{t("inspector.location.openLocation")}
					</CircleButton>
				)}

				<CircleButton
					icon={ArrowsClockwise}
					onClick={reindexMenu.show}
title={t("inspector.location.reindex")}
				/>
			</div>

			{/* Details */}
<Section title={t("inspector.location.details")} icon={Info}>
<InfoRow label={t("inspector.file.path")} value={'Physical' in location.wing_path ? location.wing_path.Physical.path : 'Cloud' in location.wing_path ? location.wing_path.Cloud.path : location.name} mono />
			{location.file_count != null && (
				<InfoRow
label={t("inspector.location.totalFiles")}
					value={location.file_count?.toLocaleString() ?? "0"}
				/>
			)}
				<InfoRow
label={t("inspector.location.totalSize")}
					value={formatBytes(location.total_size)}
				/>
<InfoRow label={t("inspector.location.scanState")} value={formatScanState(location.scan_state)} />
				{location.last_scan_at && (
					<InfoRow
label={t("inspector.location.lastScan")}
						value={formatDate(location.last_scan_at)}
					/>
				)}
			</Section>

			{/* Index Mode */}
<Section title={t("inspector.location.indexMode")} icon={Gear}>
				<InfoRow
label={t("inspector.location.mode")}
					value={
						location.index_mode.charAt(0).toUpperCase() +
						location.index_mode.slice(1)
					}
				/>
			</Section>
		</div>
	);
}

function IndexingTab({ location }: { location: Location }) {
const { t } = useTranslation("inspector");
	const [indexMode, setIndexMode] = useState<"shallow" | "content" | "deep">(
		location.index_mode as "shallow" | "content" | "deep",
	);
	const [ignoreRules, setIgnoreRules] = useState([
		".git",
		"node_modules",
		"*.tmp",
		".DS_Store",
	]);

	return (
		<div className="no-scrollbar mask-fade-out flex flex-col space-y-5 overflow-x-hidden overflow-y-scroll pb-10 px-2 pt-2">
<Section title={t("inspector.location.indexMode")} icon={Gear}>
				<p className="text-xs text-sidebar-ink-dull mb-3">
{t("inspector.location.indexDepthHint")}
				</p>

				<div className="space-y-2">
					<RadioOption
						value="shallow"
label={t("inspector.location.shallow")}
description={t("inspector.location.shallowDescription")}
						checked={indexMode === "shallow"}
						onChange={() => setIndexMode("shallow")}
					/>
					<RadioOption
						value="content"
label={t("inspector.location.content")}
description={t("inspector.location.contentDescription")}
						checked={indexMode === "content"}
						onChange={() => setIndexMode("content")}
					/>
					<RadioOption
						value="deep"
label={t("inspector.location.deep")}
description={t("inspector.location.deepDescription")}
						checked={indexMode === "deep"}
						onChange={() => setIndexMode("deep")}
					/>
				</div>
			</Section>

<Section title={t("inspector.location.ignoreRules")} icon={FunnelX}>
				<p className="text-xs text-sidebar-ink-dull mb-3">
{t("inspector.location.ignoreRulesDescription")}
				</p>

				<div className="space-y-1">
					{ignoreRules.map((pattern, i) => (
						<IgnoreRule
							key={i}
							pattern={pattern}
							onRemove={() => {
								setIgnoreRules(
									ignoreRules.filter((_, idx) => idx !== i),
								);
							}}
						/>
					))}
				</div>

				<button className="mt-2 text-xs text-accent hover:text-accent/80 transition-colors">
{t("inspector.location.addRule")}
				</button>
			</Section>
		</div>
	);
}

function JobsTab({ location }: { location: Location }) {
const { t } = useTranslation("inspector");
	const updateLocation = useLibraryMutation("locations.update");
	const triggerJob = useLibraryMutation("locations.triggerJob");

	const updatePolicy = async (
		updates: Partial<typeof location.job_policies>,
	) => {
		await updateLocation.mutateAsync({
			id: location.id,
			name: null,
			job_policies: {
				...location.job_policies,
				...updates,
			},
		});
	};

	const thumbnails = location.job_policies?.thumbnail?.enabled ?? true;
	const thumbstrips = location.job_policies?.thumbstrip?.enabled ?? true;
	const proxies = location.job_policies?.proxy?.enabled ?? false;
	const ocr = location.job_policies?.ocr?.enabled ?? false;
	const speech = location.job_policies?.speech_to_text?.enabled ?? false;

	return (
		<div className="no-scrollbar mask-fade-out flex flex-col space-y-5 overflow-x-hidden overflow-y-scroll pb-10 px-2 pt-2">
			<p className="text-xs text-sidebar-ink-dull">
{t("inspector.location.jobsDescription")}
			</p>

<Section title={t("inspector.location.mediaProcessing")} icon={Image}>
				<div className="space-y-2.5">
					<JobConfigRow
label={t("inspector.location.generateThumbnails")}
description={t("inspector.location.thumbnailsDescription")}
						enabled={thumbnails}
						onToggle={(enabled) =>
							updatePolicy({
								thumbnail: {
									sizes: [],
									quality: 80,
									regenerate: false,
									...location.job_policies?.thumbnail,
									enabled,
								},
							})
						}
						onTrigger={() =>
							triggerJob.mutate({
								location_id: location.id,
								job_type: "thumbnail",
								force: false,
							})
						}
						isTriggering={triggerJob.isPending}
					/>
					<JobConfigRow
label={t("inspector.location.generateThumbstrips")}
description={t("inspector.location.thumbstripsDescription")}
						enabled={thumbstrips}
						onToggle={(enabled) =>
							updatePolicy({
								thumbstrip: {
									regenerate: false,
									...location.job_policies?.thumbstrip,
									enabled,
								},
							})
						}
						onTrigger={() =>
							triggerJob.mutate({
								location_id: location.id,
								job_type: "thumbstrip",
								force: false,
							})
						}
						isTriggering={triggerJob.isPending}
						icon={FilmStrip}
					/>
					<JobConfigRow
label={t("inspector.location.generateProxies")}
description={t("inspector.location.proxiesDescription")}
						enabled={proxies}
						onToggle={(enabled) =>
							updatePolicy({
								proxy: {
									regenerate: false,
									...location.job_policies?.proxy,
									enabled,
								},
							})
						}
						onTrigger={() =>
							triggerJob.mutate({
								location_id: location.id,
								job_type: "thumbnail" as const,
								force: false,
							})
						}
						isTriggering={triggerJob.isPending}
						icon={VideoCamera}
					/>
				</div>
			</Section>

<Section title={t("inspector.location.aiProcessing")} icon={Sparkle}>
				<div className="space-y-2.5">
					<JobConfigRow
label={t("inspector.location.extractTextOcr")}
description={t("inspector.location.ocrDescription")}
						enabled={ocr}
						onToggle={(enabled) =>
							updatePolicy({
								ocr: {
									languages: ['eng'],
									min_confidence: 0.5,
									reprocess: false,
									...location.job_policies?.ocr,
									enabled,
								},
							})
						}
						onTrigger={() =>
							triggerJob.mutate({
								location_id: location.id,
								job_type: "ocr",
								force: false,
							})
						}
						isTriggering={triggerJob.isPending}
					/>
					<JobConfigRow
label={t("inspector.location.speechToText")}
description={t("inspector.location.speechDescription")}
						enabled={speech}
						onToggle={(enabled) =>
							updatePolicy({
								speech_to_text: {
									language: null,
									model: 'base',
									reprocess: false,
									...location.job_policies?.speech_to_text,
									enabled,
								},
							})
						}
						onTrigger={() =>
							triggerJob.mutate({
								location_id: location.id,
								job_type: "speech_to_text",
								force: false,
							})
						}
						isTriggering={triggerJob.isPending}
					/>
				</div>
			</Section>
		</div>
	);
}

function ActivityTab({ location: _location }: { location: Location }) {
	const { t } = useTranslation("inspector");
	const activity = [
		{ action: t("inspector.location.fullScanCompleted"), time: t("inspector.location.minutesAgo", { count: 10 }), files: 12456 },
		{ action: t("inspector.location.thumbnailsGenerated"), time: t("inspector.location.hoursAgo", { count: 1 }), files: 234 },
		{ action: t("inspector.location.contentHashesUpdated"), time: t("inspector.location.hoursAgo", { count: 3 }), files: 5678 },
		{ action: t("inspector.location.metadataExtracted"), time: t("inspector.location.hoursAgo", { count: 5 }), files: 890 },
		{ action: t("inspector.location.locationAdded"), time: t("inspector.location.addedDate"), files: 0 },
	];

	return (
		<div className="no-scrollbar mask-fade-out flex flex-col space-y-4 overflow-x-hidden overflow-y-scroll pb-10 px-2 pt-2">
			<p className="text-xs text-sidebar-ink-dull">
				{t("inspector.location.recentActivity")}
			</p>

			<div className="space-y-0.5">
				{activity.map((item, i) => (
					<div
						key={i}
						className="flex items-start gap-3 p-2 hover:bg-app-box/40 rounded-lg transition-colors"
					>
						<ClockCounterClockwise
							className="size-4 text-sidebar-ink-dull shrink-0 mt-0.5"
							weight="bold"
						/>
						<div className="flex-1 min-w-0">
							<div className="text-xs text-sidebar-ink">
								{item.action}
							</div>
							<div className="text-[11px] text-sidebar-ink-dull mt-0.5">
								{item.time}
								{item.files > 0 &&
									` · ${item.files.toLocaleString()} ${t("inspector.location.files")}`}
							</div>
						</div>
					</div>
				))}
			</div>
		</div>
	);
}

function DevicesTab({ location: _location }: { location: Location }) {
	const { t } = useTranslation("inspector");
	const devices = [
		{
			name: "MacBook Pro",
			status: "online" as const,
			lastSeen: t("inspector.location.minutesAgo", { count: 2 }),
		},
		{
			name: t("inspector.location.desktopPc"),
			status: "offline" as const,
			lastSeen: t("inspector.location.daysAgo", { count: 2 }),
		},
		{
			name: t("inspector.location.homeServer"),
			status: "online" as const,
			lastSeen: t("inspector.location.minutesAgo", { count: 5 }),
		},
	];

	return (
		<div className="no-scrollbar mask-fade-out flex flex-col space-y-4 overflow-x-hidden overflow-y-scroll pb-10 px-2 pt-2">
			<p className="text-xs text-sidebar-ink-dull">
				{t("inspector.location.devicesDescription")}
			</p>

			<div className="space-y-2">
				{devices.map((device, i) => (
					<div
						key={i}
						className="p-2.5 bg-app-box/40 rounded-lg border border-app-line/50"
					>
						<div className="flex items-center gap-2">
							<HardDrive
								className="size-4 text-accent"
								weight="bold"
							/>
							<div className="flex-1 min-w-0">
								<div className="text-xs font-medium text-sidebar-ink">
									{device.name}
								</div>
								<div className="text-[11px] text-sidebar-ink-dull flex items-center gap-1">
									<div
										className={clsx(
											"size-1.5 rounded-full",
											device.status === "online"
												? "bg-green-500"
												: "bg-sidebar-ink-dull",
										)}
									/>
									<span>
										{device.status === "online"
											? t("inspector.location.online")
											: t("inspector.location.offline")}{" "}
										· {device.lastSeen}
									</span>
								</div>
							</div>
						</div>
					</div>
				))}
			</div>
		</div>
	);
}

interface DeleteLocationDialogProps extends UseDialogProps {
	locationId: string;
	locationName: string;
}

function useDeleteLocationDialog() {
	return (locationId: string, locationName: string) =>
		dialogManager.create((props: UseDialogProps) => (
			<DeleteLocationDialog {...props} locationId={locationId} locationName={locationName} />
		));
}

function DeleteLocationDialog({ locationId, locationName, ...props }: DeleteLocationDialogProps) {
	const { t } = useTranslation("inspector");
	const dialog = useDialog(props);
	const form = useForm();
	const queryClient = useQueryClient();
	const removeLocation = useLibraryMutation("locations.remove", {
		onSuccess: () => {
			// Manually invalidate the locations query until the backend emits ResourceDeleted events
			// This forces a refetch so the location disappears from the sidebar immediately
			queryClient.invalidateQueries({
				predicate: (query) => {
					const key = query.queryKey;
					return Array.isArray(key) && key[0] === "query:locations.list";
				},
			});

			// Close the dialog
			dialogManager.setState(dialog.id, { open: false });
		},
	});

	const handleDelete = async () => {
		try {
			await removeLocation.mutateAsync({
				location_id: String(locationId),
			});
		} catch (error) {
			console.error("Failed to remove location:", error);
		}
	};

	return (
		<Dialog
			dialog={dialog}
			form={form}
			title={t("inspector.location.removeLocation")}
			description={t("inspector.location.removeConfirm", { name: locationName })}
			icon={<Trash className="text-red-400" weight="bold" />}
			ctaLabel={t("inspector.location.removeLocation")}
			ctaDanger
			cancelLabel={t("inspector.location.cancel")}
			cancelBtn
			onSubmit={form.handleSubmit(handleDelete)}
			loading={removeLocation.isPending}
		/>
	);
}

function MoreTab({ location }: { location: Location }) {
const { t } = useTranslation("inspector");
	const openDeleteDialog = useDeleteLocationDialog();

	const formatDate = (dateStr: string) => {
		const date = new Date(dateStr);
		return date.toLocaleDateString(uiLocale(), {
			month: "short",
			day: "numeric",
			year: "numeric",
			hour: "2-digit",
			minute: "2-digit",
		});
	};

	return (
		<div className="no-scrollbar mask-fade-out flex flex-col space-y-5 overflow-x-hidden overflow-y-scroll pb-10 px-2 pt-2">
<Section title={t("inspector.location.advanced")} icon={Gear}>
				<InfoRow
label={t("inspector.location.locationId")}
					value={String(location.id).slice(0, 8) + "..."}
					mono
				/>
				{location.created_at && (
					<InfoRow
label={t("inspector.file.created")}
						value={formatDate(location.created_at)}
					/>
				)}
				{location.last_scan_at && (
					<InfoRow
label={t("inspector.location.lastScan")}
						value={formatDate(location.last_scan_at)}
					/>
				)}
			</Section>

<Section title={t("inspector.location.dangerZone")} icon={Trash}>
				<p className="text-xs text-sidebar-ink-dull mb-3">
					{t("ui.removingThisLocationWillNotDelete")}
				</p>
				<button
					onClick={() => openDeleteDialog(location.id, location.name)}
					className="w-full px-3 py-2 bg-red-500/10 hover:bg-red-500/20 border border-red-500/30 rounded-lg text-sm font-medium text-red-400 transition-colors"
				>
					<div className="flex items-center justify-center gap-2">
						<Trash className="size-4" weight="bold" />
<span>{t("inspector.location.removeLocation")}</span>
					</div>
				</button>
			</Section>
		</div>
	);
}

// Helper Components

interface RadioOptionProps {
	value: string;
	label: string;
	description: string;
	checked: boolean;
	onChange: () => void;
}

function RadioOption({
	value: _value,
	label,
	description,
	checked,
	onChange,
}: RadioOptionProps) {
	return (
		<button
			onClick={onChange}
			className={clsx(
				"w-full p-2.5 rounded-lg border transition-colors text-left",
				checked
					? "bg-accent/10 border-accent/30"
					: "bg-app-box/40 border-app-line/50 hover:bg-app-box/60",
			)}
		>
			<div className="flex items-start gap-2">
				<div
					className={clsx(
						"size-4 rounded-full border-2 shrink-0 mt-0.5 flex items-center justify-center",
						checked ? "border-accent" : "border-sidebar-ink-dull",
					)}
				>
					{checked && (
						<div className="size-2 rounded-full bg-accent" />
					)}
				</div>
				<div className="flex-1 min-w-0">
					<div className="text-xs font-medium text-sidebar-ink">
						{label}
					</div>
					<div className="text-[11px] text-sidebar-ink-dull mt-0.5">
						{description}
					</div>
				</div>
			</div>
		</button>
	);
}

interface IgnoreRuleProps {
	pattern: string;
	onRemove: () => void;
}

function IgnoreRule({ pattern, onRemove }: IgnoreRuleProps) {
	const { t } = useTranslation("inspector");
	return (
		<div className="flex items-center gap-2 p-2 bg-app-box/40 rounded-lg border border-app-line/50 group">
			<code className="flex-1 text-xs text-sidebar-ink font-mono">
				{pattern}
			</code>
			<button
				onClick={onRemove}
				className="size-5 rounded flex items-center justify-center opacity-0 group-hover:opacity-100 hover:bg-red-500/20 transition-all"
title={t("inspector.location.removeRule")}
			>
				<X className="size-3 text-red-400" weight="bold" />
			</button>
		</div>
	);
}

interface JobConfigRowProps {
	label: string;
	description: string;
	enabled: boolean;
	onToggle: (enabled: boolean) => void;
	onTrigger: () => void;
	isTriggering: boolean;
	icon?: React.ComponentType<any>;
}

function JobConfigRow({
	label,
	description,
	enabled,
	onToggle,
	onTrigger,
	isTriggering,
	icon: Icon,
}: JobConfigRowProps) {
	const { t } = useTranslation("inspector");
	return (
		<div className="w-full p-3 bg-app-box/40 rounded-lg border border-app-line/50">
			{/* Header with toggle and icon */}
			<div className="space-y-1.5">
				<button
					onClick={() => onToggle(!enabled)}
					className="flex items-center gap-2.5 w-full text-left group"
				>
					{enabled ? (
						<ToggleRight
							className="size-5 text-accent shrink-0"
							weight="fill"
						/>
					) : (
						<ToggleLeft
							className="size-5 text-sidebar-ink-dull shrink-0 group-hover:text-sidebar-ink transition-colors"
							weight="fill"
						/>
					)}
					<div className="flex items-center gap-2 flex-1 min-w-0">
						{Icon && (
							<Icon
								className="size-4 text-sidebar-ink-dull shrink-0"
								weight="bold"
							/>
						)}
						<div className="flex-1 min-w-0">
							<div className="text-xs font-medium text-sidebar-ink">
								{label}
							</div>
						</div>
					</div>
				</button>

				{/* Description */}
				<p className="text-[11px] text-sidebar-ink-dull leading-relaxed pl-7">
					{description}
				</p>
			</div>

			{/* Run button */}
			<Button
				onClick={onTrigger}
				disabled={!enabled || isTriggering}
				variant="gray"
				size="sm"
				className="w-full flex items-center justify-center gap-1.5 mt-2.5"
title={enabled ? t("inspector.location.runJobNow") : t("inspector.location.enableJobFirst")}
			>
				<Play className="size-3" weight="fill" />
{isTriggering ? t("inspector.location.running") : t("inspector.location.runNow")}
			</Button>
		</div>
	);
}
