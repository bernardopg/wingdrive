import { motion } from "framer-motion";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "../../../i18n";

interface RedundancyVolumeBarProps {
	volumeUuid: string;
	displayName: string | null;
	totalBytes: number;
	atRiskBytes: number;
	atRiskFileCount: number;
	redundantBytes: number;
	redundantFileCount: number;
	totalFileCount: number;
}

function formatBytes(bytes: number): string {
	if (bytes === 0) return "0 B";
	const k = 1024;
	const sizes = ["B", "KB", "MB", "GB", "TB", "PB"];
	const i = Math.floor(Math.log(bytes) / Math.log(k));
	return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

export function RedundancyVolumeBar({
	volumeUuid,
	displayName,
	totalBytes,
	atRiskBytes,
	atRiskFileCount,
	redundantBytes,
	redundantFileCount,
	totalFileCount,
}: RedundancyVolumeBarProps) {
	const { t } = useTranslation("navigation");
	const navigate = useNavigate();

	const redundantPercent =
		totalBytes > 0 ? (redundantBytes / totalBytes) * 100 : 0;
	const atRiskPercent =
		totalBytes > 0 ? (atRiskBytes / totalBytes) * 100 : 0;
	// Remaining is unindexed content (no content_id)
	const unindexedPercent = Math.max(
		0,
		100 - redundantPercent - atRiskPercent,
	);

	return (
		<div className="rounded-lg border border-app-line bg-app-box/50 p-3">
			{/* Header */}
			<div className="mb-2 flex items-center justify-between">
				<span className="truncate text-sm font-medium text-ink">
					{displayName || t("items.unknownVolume")}
				</span>
				<span className="text-xs text-ink-dull">
					{t("redundancy.fileCount", {count: totalFileCount, formattedCount: totalFileCount.toLocaleString()})}
				</span>
			</div>

			{/* Bar */}
			<div className="flex h-3 w-full overflow-hidden rounded-full bg-app-box">
				{/* Redundant segment (safe) */}
				{redundantPercent > 0 && (
					<motion.div
						className="h-full cursor-pointer bg-status-success/70 hover:bg-status-success/90"
						initial={{ width: 0 }}
						animate={{ width: `${redundantPercent}%` }}
						transition={{
							duration: 0.6,
							ease: [0.25, 1, 0.5, 1],
						}}
						title={t("redundancy.redundantTitle", {size: formatBytes(redundantBytes), count: redundantFileCount})}
						onClick={() =>
							navigate(
								`/redundancy/at-risk?volume=${volumeUuid}&at_risk=false`,
							)
						}
					/>
				)}
				{/* At-risk segment */}
				{atRiskPercent > 0 && (
					<motion.div
						className="h-full cursor-pointer bg-status-warning/70 hover:bg-status-warning/90"
						initial={{ width: 0 }}
						animate={{ width: `${atRiskPercent}%` }}
						transition={{
							duration: 0.6,
							ease: [0.25, 1, 0.5, 1],
							delay: 0.1,
						}}
						title={t("redundancy.atRiskTitle", {size: formatBytes(atRiskBytes), count: atRiskFileCount})}
						onClick={() =>
							navigate(
								`/redundancy/at-risk?volume=${volumeUuid}&at_risk=true`,
							)
						}
					/>
				)}
				{/* Unindexed segment */}
				{unindexedPercent > 0 && (
					<motion.div
						className="h-full bg-app-line/50"
						initial={{ width: 0 }}
						animate={{ width: `${unindexedPercent}%` }}
						transition={{
							duration: 0.6,
							ease: [0.25, 1, 0.5, 1],
							delay: 0.2,
						}}
					/>
				)}
			</div>

			{/* Legend */}
			<div className="mt-2 flex gap-4 text-xs text-ink-dull">
				<div className="flex items-center gap-1.5">
					<div className="size-2 rounded-full bg-status-success/70" />
					<span>
						{t("redundancy.redundantLabel", {size: formatBytes(redundantBytes)})}
					</span>
				</div>
				<div className="flex items-center gap-1.5">
					<div className="size-2 rounded-full bg-status-warning/70" />
					<span>
						{t("redundancy.atRiskLabel", {size: formatBytes(atRiskBytes)})}
					</span>
				</div>
			</div>
		</div>
	);
}
