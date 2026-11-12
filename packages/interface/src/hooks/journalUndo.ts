import type {JobResult} from './useWaitForJob';

/** Why an undo job did not finish its work, already worded for a person. */
export type UndoMessages = {
	skipped: (count: number, reason: string) => string;
	cancelled: () => string;
};

/**
 * Builds the undo step for operations the core journaled (`files.undo`).
 *
 * Operations are undone newest first. Each one leaves the list once its undo
 * job reversed everything, so running the step again after a failure retries
 * only what is left instead of asking the core to undo an operation twice.
 * An undo job that skipped items throws: the store keeps the entry, and the
 * skipped items stay in the journal for the next try.
 */
export function journaledUndo(
	ids: readonly string[],
	undoOne: (id: string) => Promise<JobResult>,
	messages: UndoMessages
): () => Promise<void> {
	const remaining = [...ids];
	return async () => {
		while (remaining.length > 0) {
			const id = remaining[remaining.length - 1]!;
			const result = await undoOne(id);
			if (result.status === 'failed') throw new Error(result.error);
			if (result.status === 'cancelled') throw new Error(messages.cancelled());
			if (
				result.status === 'completed' &&
				result.output.type === 'FileUndo' &&
				result.output.data.skipped_count > 0
			) {
				throw new Error(
					messages.skipped(
						result.output.data.skipped_count,
						result.output.data.skipped?.[0] ?? ''
					)
				);
			}
			// A timeout means the job is still running; it finishes on its own.
			remaining.pop();
		}
	};
}
