import {describe, expect, it} from 'bun:test';
import {
	combineFileOperations,
	summarizeFileOperation
} from './fileOperationOutcome';

function copied(copied_count: number, failed_count = 0, skipped_count = 0) {
	return {
		status: 'completed' as const,
		output: {
			type: 'FileCopy' as const,
			data: {
				copied_count,
				failed_count,
				skipped_count,
				total_bytes: 0,
				errors: failed_count > 0 ? ['/a/x.txt: Permission denied'] : []
			}
		}
	};
}

describe('summarizeFileOperation', () => {
	it('reports a full copy as success', () => {
		const outcome = summarizeFileOperation('copy', copied(3));
		expect(outcome.status).toBe('success');
		expect(outcome.message).toBe('Copied 3 items');
	});

	it('never reports a copy with failures as success', () => {
		const outcome = summarizeFileOperation('copy', copied(2, 1));
		expect(outcome.status).toBe('partial');
		expect(outcome.message).toBe('Copied 2 items, 1 failed');
		expect(outcome.details).toEqual(['/a/x.txt: Permission denied']);
	});

	it('keeps skipped items visible', () => {
		const outcome = summarizeFileOperation('copy', copied(1, 0, 2));
		expect(outcome.status).toBe('partial');
		expect(outcome.skipped).toBe(2);
		expect(outcome.message).toBe('Copied 1 item, 2 skipped');
	});

	it('treats a run where nothing arrived as failed', () => {
		const outcome = summarizeFileOperation('duplicate', copied(0, 2));
		expect(outcome.status).toBe('failed');
	});

	it('reads move counts', () => {
		const outcome = summarizeFileOperation('move', {
			status: 'completed',
			output: {
				type: 'FileMove',
				data: {
					moved_count: 4,
					failed_count: 1,
					skipped_count: 0,
					total_bytes: 0,
					errors: []
				}
			}
		});
		expect(outcome.status).toBe('partial');
		expect(outcome.message).toBe('Moved 4 items, 1 failed');
	});

	it('reads delete counts', () => {
		const outcome = summarizeFileOperation('delete', {
			status: 'completed',
			output: {
				type: 'FileDelete',
				data: {deleted_count: 1, failed_count: 1, total_bytes: 0}
			}
		});
		expect(outcome.status).toBe('partial');
	});

	it('explains a cancel without claiming a rollback', () => {
		const outcome = summarizeFileOperation('copy', {status: 'cancelled'});
		expect(outcome.status).toBe('cancelled');
		expect(outcome.message).toContain('stay where they are');
	});

	it('reports a job that outlived the wait as running, not failed', () => {
		const outcome = summarizeFileOperation('copy', {status: 'timeout'});
		expect(outcome.status).toBe('running');
	});

	it('passes the job error through', () => {
		const outcome = summarizeFileOperation('move', {
			status: 'failed',
			error: 'Cannot copy to /mnt: not enough space'
		});
		expect(outcome.status).toBe('failed');
		expect(outcome.message).toContain('not enough space');
	});

	it('accepts jobs without counts as success', () => {
		const outcome = summarizeFileOperation('copy', {
			status: 'completed',
			output: {type: 'Success'}
		});
		expect(outcome.status).toBe('success');
	});
});

describe('combineFileOperations', () => {
	it('adds up per-item jobs', () => {
		const outcome = combineFileOperations('duplicate', [
			summarizeFileOperation('duplicate', copied(1)),
			summarizeFileOperation('duplicate', copied(0, 1))
		]);
		expect(outcome.status).toBe('partial');
		expect(outcome.message).toBe('Duplicated 1 item, 1 failed');
	});

	it('counts a failed job without counts as one failure', () => {
		const outcome = combineFileOperations('duplicate', [
			summarizeFileOperation('duplicate', copied(1)),
			summarizeFileOperation('duplicate', {status: 'failed', error: 'boom'})
		]);
		expect(outcome.failed).toBe(1);
		expect(outcome.details).toEqual(['Could not duplicate: boom']);
	});

	it('stays success when every job succeeded', () => {
		const outcome = combineFileOperations('duplicate', [
			summarizeFileOperation('duplicate', copied(1)),
			summarizeFileOperation('duplicate', copied(1))
		]);
		expect(outcome.status).toBe('success');
		expect(outcome.done).toBe(2);
	});
});
