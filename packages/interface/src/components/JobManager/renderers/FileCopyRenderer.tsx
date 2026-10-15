import { Pause, Play, X, CaretDown } from "@phosphor-icons/react";
import { motion } from "framer-motion";
import type { JobRenderer, JobRendererProps, JobDetailsRendererProps } from "./index";
import { CopyJobDetails } from "../components/CopyJobDetails";
import { useNormalizedQuery } from "../../../contexts/WingDriveContext";
import type { Device } from "@wingdrive/ts-client";
import { i18n, useTranslation } from "../../../i18n";

/**
 * Map strategy name to display label (enables i18n in future)
 */
function getStrategyLabel(strategyName: string | undefined, _isMove: boolean): string | null {
	if (!strategyName) return null;

	switch (strategyName) {
		case "RemoteTransfer":
			return i18n.t("dialogs:jobs.strategyNetwork");
		case "LocalMove":
			return i18n.t("dialogs:jobs.strategyAtomic");
		case "FastCopy":
			return i18n.t("dialogs:jobs.strategyFast");
		case "LocalStream":
			return i18n.t("dialogs:jobs.strategyStreaming");
		default:
			return strategyName;
	}
}

/**
 * Extract first filename from copy job for display
 */
function extractFirstFileName(job: JobRendererProps["job"]): string {
	const actionInput = job.action_context?.action_input;
	if (
		typeof actionInput === "object" &&
		actionInput !== null &&
		"sources" in actionInput
	) {
		const sources = actionInput.sources;
		if (
			typeof sources === "object" &&
			sources !== null &&
			"paths" in sources &&
			Array.isArray(sources.paths) &&
			sources.paths.length > 0
		) {
			const firstPath = sources.paths[0];
			if (
				typeof firstPath === "object" &&
				firstPath !== null &&
				"Physical" in firstPath
			) {
				const physical = firstPath.Physical;
				if (typeof physical === "object" && physical !== null && "path" in physical) {
					const path = String(physical.path);
					return path.split("/").pop() || path;
				}
			}
		}
	}
	return "items";
}

/**
 * Format bytes to human readable
 */
function formatBytes(bytes: number): string {
	const units = ["B", "KB", "MB", "GB", "TB"];
	let size = bytes;
	let unitIndex = 0;

	while (size >= 1024 && unitIndex < units.length - 1) {
		size /= 1024;
		unitIndex++;
	}

	return `${size.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
}

/**
 * Format speed (bytes/sec) to human readable
 */
function formatSpeed(bytesPerSecond: number): string {
	return `${formatBytes(bytesPerSecond)}/s`;
}

/**
 * Format duration from seconds to human readable
 */
function formatDurationSeconds(seconds: number): string {
	const mins = Math.floor(seconds / 60);
	const hrs = Math.floor(mins / 60);

	if (hrs > 0) {
		return i18n.t("dialogs:jobs.durationHours", { hours: hrs, minutes: mins % 60 });
	}
	if (mins > 0) {
		return i18n.t("dialogs:jobs.durationMinutesOnly", { minutes: mins });
	}
	return i18n.t("dialogs:jobs.durationSeconds", { seconds: Math.floor(seconds) });
}

/**
 * Custom card content for file copy jobs
 * Renders the full card content: title row + subtext row
 */
function FileCopyCardContent({
	job,
	isExpanded,
	statusBadge,
	canExpand,
	isHovered,
	showActionButton,
	canPause,
	canResume,
	canCancel,
	onAction,
	onCancel,
}: JobRendererProps) {
	const { t } = useTranslation("dialogs");
	const generic = job.generic_progress;
	const metadata = (generic as any)?.metadata as any;
	const strategyName = metadata?.strategy?.strategy_name;
	const strategyLabel = getStrategyLabel(strategyName, job.action_context?.action_type === "files.move");

	// Fetch devices to determine if destination is remote
	const { data: devices } = useNormalizedQuery<any, Device[]>({
		query: "devices.list",
		input: { include_offline: true, include_details: false },
		resourceType: "device",
	});

	// Determine if this is a move operation
	const isMove = job.action_context?.action_type === "files.move";

	// Check if this is a cross-device transfer from metadata
	const isCrossDevice = metadata?.strategy?.is_cross_device === true;

	// Find current device and infer destination device
	const currentDevice = devices?.find(d => d.is_current);

	// For cross-device transfers, infer the destination device
	// LIMITATION: This assumes only 2 devices in the transfer scenario.
	// TODO: Add destination_device_id to CopyStrategyMetadata in Rust
	// (core/src/ops/files/copy/routing.rs) to properly identify the destination
	// device in multi-device environments instead of inferring it.
	const destinationDevice = isCrossDevice && currentDevice
		? devices?.find(d => !d.is_current)
		: null;

	// Calculate title
	const fileCount = generic?.completion?.total || 0;
	const fileName = extractFirstFileName(job);
	const baseTitle = fileCount > 1
		? t(isMove ? "jobs.movingCount" : "jobs.copyingCount", { count: fileCount })
		: t(isMove ? "jobs.movingFile" : "jobs.copyingFile", { name: fileName });

	const title = destinationDevice
		? `${baseTitle} to ${destinationDevice.name}`
		: baseTitle;

	// Calculate rich subtext with progress, speed, and ETA
	const completed = generic?.completion?.completed || 0;
	const speed = generic?.performance?.rate ? formatSpeed(generic.performance.rate) : null;
	const eta = generic?.performance?.estimated_remaining
		? formatDurationSeconds(generic.performance.estimated_remaining.secs)
		: null;

	const subtext =
		job.status === "running" && speed && eta
			? t("jobs.copyProgress", { completed, total: fileCount, speed, eta })
			: job.status === "running"
			? t("jobs.filesProgress", { completed, total: fileCount })
			: job.status === "completed"
			? t("jobs.completed")
			: job.status === "failed"
			? t("jobs.failed")
			: job.status === "paused"
			? t("jobs.paused")
			: t("jobs.preparing");

	return (
		<>
			{/* Row 1: Title, badges, status badge, and controls */}
			<div className="flex items-center gap-3 min-h-0">
				<span className="flex-1 truncate text-[13px] font-medium text-ink">
					{title}
				</span>

				{/* Transfer method badge */}
				{strategyLabel && (
					<span className="px-1.5 py-0.5 text-[9px] font-medium text-ink-faint bg-app-dark-box rounded-md whitespace-nowrap">
						{strategyLabel}
					</span>
				)}

				{/* Status badge */}
				<span className="flex-shrink-0 text-[11px] font-medium text-ink-dull max-w-[80px] truncate">
					{statusBadge}
				</span>

				{/* Expansion caret */}
				{canExpand && (
					<motion.div
						animate={{ rotate: isExpanded ? 180 : 0 }}
						transition={{ duration: 0.15, ease: [0.25, 1, 0.5, 1] }}
						className="flex-shrink-0"
					>
						<CaretDown size={12} weight="bold" className="text-ink-dull" />
					</motion.div>
				)}

				{/* Action buttons */}
				{isHovered && (
					<div className="flex items-center gap-1">
						{showActionButton && (canPause || canResume) && (
							<button
								onClick={onAction}
								className="flex-shrink-0 flex items-center justify-center w-4 h-4 rounded-full bg-app-hover hover:bg-app-selected transition-colors"
								title={canPause ? t("jobs.pause") : t("jobs.resume")}
							>
								{canPause ? (
									<Pause size={10} weight="fill" className="text-ink" />
								) : (
									<Play size={10} weight="fill" className="text-ink" />
								)}
							</button>
						)}
						{canCancel && (
							<button
								onClick={onCancel}
								className="flex-shrink-0 flex items-center justify-center w-4 h-4 rounded-full bg-app-hover hover:bg-red-500 transition-colors"
								title={t("jobs.cancel")}
							>
								<X size={10} weight="bold" className="text-ink hover:text-white" />
							</button>
						)}
					</div>
				)}
			</div>

			{/* Row 2: Rich subtext */}
			<div className="min-h-0">
				<span
					className="text-[10px] text-ink-dull max-w-[200px] truncate block"
					style={{ opacity: 0.7 }}
				>
					{subtext}
				</span>
			</div>
		</>
	);
}

/**
 * Details panel for expanded file copy jobs
 */
function FileCopyDetailsPanel({ job, speedHistory }: JobDetailsRendererProps) {
	return <CopyJobDetails job={job} speedHistory={speedHistory} />;
}

/**
 * File copy job renderer with rich UI
 */
export const FileCopyRenderer: JobRenderer = {
	CardContent: FileCopyCardContent,
	DetailsPanel: FileCopyDetailsPanel,
};
