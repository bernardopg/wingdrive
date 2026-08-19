import { useCallback } from "react";
import type { Event, JobOutput } from "@sd/ts-client";
import { useSpacedriveClient } from "../contexts/SpacedriveContext";

/**
 * # Waiting on a dispatched job
 *
 * File actions return a `JobReceipt` as soon as the job is queued, so awaiting
 * the mutation only tells you the daemon accepted the request. Anything that
 * has to react to the result (refreshing a listing, reporting a failure) needs
 * the terminal job event instead.
 *
 * The wait is bounded: a job that never reports back resolves as `timeout` so
 * callers degrade to "assume it worked" instead of hanging forever.
 */

export type JobResult =
	| { status: "completed"; output: JobOutput }
	| { status: "failed"; error: string }
	| { status: "cancelled" }
	| { status: "timeout" };

const DEFAULT_TIMEOUT_MS = 30_000;

export function useWaitForJob() {
	const client = useSpacedriveClient();

	return useCallback(
		(jobId: string, timeoutMs = DEFAULT_TIMEOUT_MS): Promise<JobResult> => {
			return new Promise((resolve) => {
				let settled = false;
				let unsubscribe: (() => void) | undefined;
				let timer: ReturnType<typeof setTimeout> | undefined;

				const settle = (result: JobResult) => {
					if (settled) return;
					settled = true;
					if (timer) clearTimeout(timer);
					unsubscribe?.();
					resolve(result);
				};

				const handleEvent = (event: Event) => {
					if (typeof event !== "object" || event === null) return;

					if ("JobCompleted" in event) {
						const data = event.JobCompleted;
						if (data.job_id === jobId) {
							settle({ status: "completed", output: data.output });
						}
						return;
					}

					if ("JobFailed" in event) {
						const data = event.JobFailed;
						if (data.job_id === jobId) {
							settle({ status: "failed", error: data.error });
						}
						return;
					}

					if ("JobCancelled" in event) {
						if (event.JobCancelled.job_id === jobId) {
							settle({ status: "cancelled" });
						}
					}
				};

				client
					.subscribeFiltered(
						{
							event_types: [
								"JobCompleted",
								"JobFailed",
								"JobCancelled",
							],
						},
						handleEvent,
					)
					.then((unsub) => {
						if (settled) {
							unsub();
							return;
						}
						unsubscribe = unsub;
					})
					.catch(() => settle({ status: "timeout" }));

				timer = setTimeout(() => settle({ status: "timeout" }), timeoutMs);
			});
		},
		[client],
	);
}
