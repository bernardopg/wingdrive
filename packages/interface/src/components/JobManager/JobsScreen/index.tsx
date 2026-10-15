import { X, FunnelSimple } from "@phosphor-icons/react";
import { CircleButton } from "@wingdrive/primitives";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useJobsContext } from "../hooks/JobsContext";
import { JobRow } from "./JobRow";
import { useTranslation } from "../../../i18n";

export function JobsScreen() {
	const { t } = useTranslation("dialogs");
	const navigate = useNavigate();
	const { jobs, pause, resume, cancel } = useJobsContext();
	const [showOnlyRunning, setShowOnlyRunning] = useState(false);

	// Newest first: the daemon returns history in insertion order
	const byRecency = (a: { created_at?: string | null; completed_at?: string | null }, b: typeof a) =>
		new Date(b.completed_at ?? b.created_at ?? 0).getTime() -
		new Date(a.completed_at ?? a.created_at ?? 0).getTime();

	// Filter jobs based on toggle
	const filteredJobs = showOnlyRunning
		? jobs.filter(
				(job) => job.status === "running" || job.status === "paused",
			)
		: [...jobs].sort(byRecency);

	// Group jobs by status
	const runningJobs = filteredJobs.filter((j) => j.status === "running");
	const pausedJobs = filteredJobs.filter((j) => j.status === "paused");
	const queuedJobs = filteredJobs.filter((j) => j.status === "queued");
	const completedJobs = filteredJobs.filter((j) => j.status === "completed");
	const failedJobs = filteredJobs.filter((j) => j.status === "failed");

	return (
		<div className="flex flex-col h-screen bg-app">
			{/* Header */}
			<div className="sticky top-0 z-10 backdrop-blur-xl bg-app/80 border-b border-app-line">
				<div className="flex items-center justify-between px-6 py-4">
					<div className="flex items-center gap-4">
						<h1 className="text-2xl font-bold text-ink">{t("jobs.jobs")}</h1>
						<div className="flex items-center gap-2 text-sm text-ink-dull">
							<span>{jobs.length} total</span>
							{runningJobs.length > 0 && (
								<>
									<span>•</span>
									<span>{runningJobs.length} running</span>
								</>
							)}
						</div>
					</div>

					<div className="flex items-center gap-2">
						{/* Filter toggle */}
						<CircleButton
							icon={FunnelSimple}
							active={showOnlyRunning}
							onClick={() => setShowOnlyRunning(!showOnlyRunning)}
							title={
								showOnlyRunning
									? t("jobs.showAll")
									: t("jobs.showActive")
							}
						/>

						{/* Back button */}
						<CircleButton
							icon={X}
							onClick={() => navigate(-1)}
							title={t("common.goBack")}
						/>
					</div>
				</div>

				{/* Column headers */}
				<div className="flex items-center gap-4 px-4 py-2 text-xs font-medium text-ink-dull uppercase tracking-wide bg-app-box/30 border-t border-app-line/30">
					<div className="flex-shrink-0 w-10" /> {/* Icon spacer */}
					<div className="flex-1 min-w-0 flex items-center gap-6">
						<div className="flex-1">{t("jobs.name")}</div>
						<div className="flex-shrink-0 w-32">{t("jobs.duration")}</div>
						<div className="flex-shrink-0 w-24 text-right">
							{t("ui.time")}
						</div>
						<div className="flex-shrink-0 w-20 text-right">
							{t("ui.status")}
						</div>
					</div>
					<div className="flex-shrink-0 w-6" />{" "}
					{/* Action button spacer */}
				</div>
			</div>

			{/* Content */}
			<div className="flex-1 overflow-y-auto">
				{filteredJobs.length === 0 ? (
					<div className="flex items-center justify-center h-full">
						<div className="text-center">
							<p className="text-sm text-ink-dull">
								{t("ui.noJobsFound")}
							</p>
						</div>
					</div>
				) : (
					<div>
						{/* Running Jobs */}
						{runningJobs.length > 0 && (
							<JobSection
								title={t("jobs.running")}
								count={runningJobs.length}
							>
								{runningJobs.map((job) => (
									<JobRow
										key={job.id}
										job={job}
										onPause={pause}
										onResume={resume}
										onCancel={cancel}
									/>
								))}
							</JobSection>
						)}

						{/* Paused Jobs */}
						{pausedJobs.length > 0 && (
							<JobSection
								title={t("jobs.paused")}
								count={pausedJobs.length}
							>
								{pausedJobs.map((job) => (
									<JobRow
										key={job.id}
										job={job}
										onPause={pause}
										onResume={resume}
										onCancel={cancel}
									/>
								))}
							</JobSection>
						)}

						{/* Queued Jobs */}
						{queuedJobs.length > 0 && (
							<JobSection
								title={t("jobs.queued")}
								count={queuedJobs.length}
							>
								{queuedJobs.map((job) => (
									<JobRow
										key={job.id}
										job={job}
										onPause={pause}
										onResume={resume}
										onCancel={cancel}
									/>
								))}
							</JobSection>
						)}

						{/* Completed Jobs */}
						{completedJobs.length > 0 && (
							<JobSection
								title={t("jobs.completed")}
								count={completedJobs.length}
							>
								{completedJobs.map((job) => (
									<JobRow
										key={job.id}
										job={job}
										onPause={pause}
										onResume={resume}
										onCancel={cancel}
									/>
								))}
							</JobSection>
						)}

						{/* Failed Jobs */}
						{failedJobs.length > 0 && (
							<JobSection
								title={t("jobs.failed")}
								count={failedJobs.length}
							>
								{failedJobs.map((job) => (
									<JobRow
										key={job.id}
										job={job}
										onPause={pause}
										onResume={resume}
										onCancel={cancel}
									/>
								))}
							</JobSection>
						)}
					</div>
				)}
			</div>
		</div>
	);
}

interface JobSectionProps {
	title: string;
	count: number;
	children: React.ReactNode;
}

function JobSection({ title, count, children }: JobSectionProps) {
	return (
		<div>
			<div className="sticky top-0 z-10 flex items-center gap-2 px-4 py-2 bg-app-box/50 backdrop-blur-sm border-b border-app-line/50">
				<h2 className="text-xs font-semibold text-ink uppercase tracking-wide">
					{title}
				</h2>
				<span className="text-xs text-ink-dull">({count})</span>
			</div>
			<div>{children}</div>
		</div>
	);
}
