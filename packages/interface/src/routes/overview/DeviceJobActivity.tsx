import { CircleNotch } from "@phosphor-icons/react";
import type { JobListItem } from "@wingdrive/ts-client";
import { useTranslation } from "../../i18n";

interface DeviceJobActivityProps {
	jobs: JobListItem[];
}

export function DeviceJobActivity({ jobs }: DeviceJobActivityProps) {
	const { t } = useTranslation("navigation");
	const activeJobs = jobs.filter(
		(j) => j.status === "running" || j.status === "paused"
	);

	if (activeJobs.length === 0) {
		return null;
	}

	const firstJob = activeJobs[0];
	const remainingCount = activeJobs.length - 1;

	return (
		<div className="flex items-center gap-2 ml-auto">
			<div className="flex items-center gap-1.5 px-2 py-1 bg-accent/10 text-accent text-xs rounded-md border border-accent/20">
				<CircleNotch size={12} className="animate-spin" weight="bold" />
				<span className="font-medium">
					{firstJob.name}
				</span>
				<span className="text-accent/70">
					{Math.round(firstJob.progress * 100)}%
				</span>
			</div>
			{remainingCount > 0 && (
				<span className="text-xs text-ink-dull">
					{t("jobs.moreCount", {count: remainingCount})}
				</span>
			)}
		</div>
	);
}
