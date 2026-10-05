import type {JobResult} from './useWaitForJob';

export type FileOperationKind = 'copy' | 'move' | 'duplicate' | 'delete';

/**
 * How a file job ended, as the user should read it.
 *
 * A job that completes can still have failed or skipped items, so "completed"
 * alone never means every item arrived. `partial` covers those runs, and
 * `running` covers a job that outlived the wait: it was not lost, it is still
 * working in the background.
 */
export type FileOperationStatus =
	'success' | 'partial' | 'failed' | 'cancelled' | 'running';

export interface FileOperationOutcome {
	status: FileOperationStatus;
	message: string;
	/** Per-item failure messages reported by the job, when it sent any. */
	details: string[];
	done: number;
	failed: number;
	skipped: number;
}

const PAST_TENSE: Record<FileOperationKind, string> = {
	copy: 'Copied',
	move: 'Moved',
	duplicate: 'Duplicated',
	delete: 'Deleted'
};

const NOUN: Record<FileOperationKind, string> = {
	copy: 'copy',
	move: 'move',
	duplicate: 'duplicate',
	delete: 'delete'
};

function items(count: number): string {
	return `${count} item${count === 1 ? '' : 's'}`;
}

function counts(result: Extract<JobResult, {status: 'completed'}>) {
	const output = result.output;
	switch (output.type) {
		case 'FileCopy':
			return {
				done: output.data.copied_count,
				failed: output.data.failed_count ?? 0,
				skipped: output.data.skipped_count ?? 0,
				details: output.data.errors ?? []
			};
		case 'FileMove':
			return {
				done: output.data.moved_count,
				failed: output.data.failed_count,
				skipped: output.data.skipped_count ?? 0,
				details: output.data.errors ?? []
			};
		case 'FileDelete':
			return {
				done: output.data.deleted_count,
				failed: output.data.failed_count,
				skipped: 0,
				details: []
			};
		default:
			return null;
	}
}

export function summarizeFileOperation(
	kind: FileOperationKind,
	result: JobResult
): FileOperationOutcome {
	const empty = {details: [], done: 0, failed: 0, skipped: 0};

	switch (result.status) {
		case 'failed':
			return {
				...empty,
				status: 'failed',
				message: `Could not ${NOUN[kind]}: ${result.error}`
			};
		case 'cancelled':
			return {
				...empty,
				status: 'cancelled',
				message:
					kind === 'delete'
						? 'Delete cancelled. Items processed before the cancel are already gone.'
						: `${PAST_TENSE[kind]} items stay where they are; the rest was cancelled.`
			};
		case 'timeout':
			return {
				...empty,
				status: 'running',
				message: `The ${NOUN[kind]} is still running in the background. Follow it in Jobs.`
			};
		case 'completed':
			break;
	}

	const tally = counts(result);
	if (!tally || (tally.failed === 0 && tally.skipped === 0)) {
		return {
			...empty,
			...tally,
			details: [],
			status: 'success',
			message: tally
				? `${PAST_TENSE[kind]} ${items(tally.done)}`
				: `${PAST_TENSE[kind]}`
		};
	}

	const parts = [`${PAST_TENSE[kind]} ${items(tally.done)}`];
	if (tally.failed > 0) parts.push(`${tally.failed} failed`);
	if (tally.skipped > 0) parts.push(`${tally.skipped} skipped`);

	return {
		...tally,
		status: tally.done === 0 && tally.failed > 0 ? 'failed' : 'partial',
		message: parts.join(', ')
	};
}

/**
 * Merges the outcomes of several jobs started by one user action (duplicate
 * runs one copy job per item) into the single result the user sees.
 */
export function combineFileOperations(
	kind: FileOperationKind,
	outcomes: FileOperationOutcome[]
): FileOperationOutcome {
	if (outcomes.length === 1) return outcomes[0];

	const done = outcomes.reduce((sum, o) => sum + o.done, 0);
	const failed = outcomes.reduce(
		(sum, o) => sum + o.failed + (o.status === 'failed' && o.failed === 0 ? 1 : 0),
		0
	);
	const skipped = outcomes.reduce((sum, o) => sum + o.skipped, 0);
	const details = outcomes.flatMap((o) =>
		o.details.length > 0
			? o.details
			: o.status === 'failed'
				? [o.message]
				: []
	);
	const running = outcomes.some((o) => o.status === 'running');
	const cancelled = outcomes.some((o) => o.status === 'cancelled');

	const parts = [`${PAST_TENSE[kind]} ${items(done)}`];
	if (failed > 0) parts.push(`${failed} failed`);
	if (skipped > 0) parts.push(`${skipped} skipped`);
	if (running) parts.push('some still running in Jobs');
	if (cancelled) parts.push('some cancelled');

	const clean = failed === 0 && skipped === 0 && !running && !cancelled;
	return {
		status: clean
			? 'success'
			: done === 0 && failed > 0 && !running
				? 'failed'
				: 'partial',
		message: parts.join(', '),
		details,
		done,
		failed,
		skipped
	};
}
