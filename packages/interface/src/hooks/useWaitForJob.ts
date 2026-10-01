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
 * callers degrade to "assume it worked" instead of hanging forever. If the
 * dispatch itself throws, the error propagates to the caller.
 */

export type JobResult =
	| { status: "completed"; output: JobOutput }
	| { status: "failed"; error: string }
	| { status: "cancelled" }
	| { status: "timeout" };

const DEFAULT_TIMEOUT_MS = 30_000;

type TerminalEvent = { jobId: string; result: JobResult };

function terminalEvent(event: Event): TerminalEvent | null {
	if (typeof event !== "object" || event === null) return null;
	if ("JobCompleted" in event) {
		const data = event.JobCompleted;
		return {
			jobId: data.job_id,
			result: { status: "completed", output: data.output },
		};
	}
	if ("JobFailed" in event) {
		const data = event.JobFailed;
		return { jobId: data.job_id, result: { status: "failed", error: data.error } };
	}
	if ("JobCancelled" in event) {
		return {
			jobId: event.JobCancelled.job_id,
			result: { status: "cancelled" },
		};
	}
	return null;
}

/**
 * Dispatches a job and waits for its terminal event.
 *
 * The subscription opens before `dispatch` runs: fast jobs (a single rename
 * or trash) finish within milliseconds, and subscribing after the receipt
 * arrived missed their event, so callers sat on the timeout. Events that
 * arrive before the receipt are buffered and matched once the id is known.
 */
export function useWaitForJob() {
	const client = useSpacedriveClient();

	return useCallback(
		async <R extends { id: string }>(
			dispatch: () => Promise<R>,
			timeoutMs = DEFAULT_TIMEOUT_MS,
		): Promise<{ receipt: R; result: JobResult }> => {
			const seen = new Map<string, JobResult>();
			let jobId: string | null = null;
			let settle: ((result: JobResult) => void) | null = null;

			const unsubscribe = await client
				.subscribeFiltered(
					{
						event_types: ["JobCompleted", "JobFailed", "JobCancelled"],
					},
					(event: Event) => {
						const terminal = terminalEvent(event);
						if (!terminal) return;
						if (jobId === null) {
							seen.set(terminal.jobId, terminal.result);
						} else if (terminal.jobId === jobId) {
							settle?.(terminal.result);
						}
					},
				)
				.catch(() => null);

			try {
				const receipt = await dispatch();
				jobId = receipt.id;
				const early = seen.get(receipt.id);
				if (early) return { receipt, result: early };
				if (!unsubscribe) return { receipt, result: { status: "timeout" } };

				const result = await new Promise<JobResult>((resolve) => {
					const timer = setTimeout(
						() => resolve({ status: "timeout" }),
						timeoutMs,
					);
					settle = (r) => {
						clearTimeout(timer);
						resolve(r);
					};
				});
				return { receipt, result };
			} finally {
				unsubscribe?.();
			}
		},
		[client],
	);
}
