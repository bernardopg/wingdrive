import {sounds} from '@wingdrive/assets/sounds';
import {useJobs as useJobsCore, type UseJobsReturn} from '@wingdrive/ts-client';
import {useVolumeIndexingStore} from '../../../stores/volumeIndexingStore';
import {useContext, useEffect, useRef} from 'react';
import {PlatformContext} from '../../../contexts/PlatformContext';
import {i18n} from '../../../i18n';
import {
	buildJobNotification,
	isWindowFocused,
	shouldNotifyForJob,
	type JobOutcome
} from '../jobNotification';
import {getJobDisplayName, type JobListItem} from '../types';

// Global set to track which jobs have already played their completion sound
// This prevents multiple hook instances from playing the sound multiple times
const completedJobSounds = new Set<string>();

// Global throttle to prevent multiple sounds within 5 seconds
let lastSoundPlayedAt = 0;
const SOUND_THROTTLE_MS = 5000;

// Every useJobs instance sees the same event; notify once per job.
const notifiedJobs = new Set<string>();

/**
 * Desktop-specific wrapper around useJobs that adds:
 * - Sound system for job completion
 * - Volume indexing tracking integration
 */
export function useJobs(): UseJobsReturn {
	const {setVolumeJob, clearVolumeJob} = useVolumeIndexingStore();
	const platform = useContext(PlatformContext);
	const jobsRef = useRef<JobListItem[]>([]);

	const notifyJobEnded = (jobId: string, outcome: JobOutcome) => {
		if (!platform?.showDesktopNotification || notifiedJobs.has(jobId)) return;
		const job = jobsRef.current.find((item) => item.id === jobId);
		if (!job || !shouldNotifyForJob(job.action_context?.action_type, isWindowFocused())) {
			return;
		}
		notifiedJobs.add(jobId);
		setTimeout(() => notifiedJobs.delete(jobId), 60_000);
		const request = buildJobNotification(outcome, getJobDisplayName(job), (key) =>
			i18n.t(key)
		);
		platform.showDesktopNotification(request).catch(() => {});
	};

	const jobs = useJobsCore({
		onJobFailed: (jobId) => notifyJobEnded(jobId, 'failed'),
		onJobCompleted: (jobId, jobType) => {
			notifyJobEnded(jobId, 'completed');
			// Play completion sound
			if (!completedJobSounds.has(jobId)) {
				completedJobSounds.add(jobId);

				// Throttle: only play sound if enough time has passed since last sound
				const now = Date.now();
				if (now - lastSoundPlayedAt >= SOUND_THROTTLE_MS) {
					lastSoundPlayedAt = now;

					// Play job-specific sound
					if (jobType?.includes('copy') || jobType?.includes('Copy')) {
						sounds.copy();
					} else {
						sounds.jobDone();
					}
				}

				// Clean up old entries after 5 seconds to prevent memory leak
				setTimeout(() => completedJobSounds.delete(jobId), 5000);
			}
		}
	});

	// Ref for stable jobs access
	useEffect(() => {
		jobsRef.current = jobs.jobs;
	}, [jobs.jobs]);

	// Track volume indexing jobs
	useEffect(() => {
		jobs.jobs.forEach((job) => {
			if (job.name === 'indexer' && job.status === 'running') {
				const context = job.action_context?.context as any;
				const volumeFingerprint = context?.volume_fingerprint;
				if (volumeFingerprint && typeof volumeFingerprint === 'string') {
					setVolumeJob(volumeFingerprint, job.id);
				}
			}
		});
	}, [jobs.jobs, setVolumeJob]);

	// Clear volume jobs on completion/failure/cancellation
	useEffect(() => {
		// This effect watches for jobs that are no longer in the list or are no longer running
		const runningIndexerJobs = jobs.jobs.filter(
			(j) => j.name === 'indexer' && j.status === 'running'
		);

		// Find any volume jobs that need to be cleared
		jobsRef.current.forEach((prevJob) => {
			if (
				prevJob.name === 'indexer' &&
				!runningIndexerJobs.find((j) => j.id === prevJob.id)
			) {
				const context = prevJob.action_context?.context as any;
				const volumeFingerprint = context?.volume_fingerprint;
				if (volumeFingerprint && typeof volumeFingerprint === 'string') {
					clearVolumeJob(volumeFingerprint);
				}
			}
		});
	}, [jobs.jobs, clearVolumeJob]);

	return jobs;
}
