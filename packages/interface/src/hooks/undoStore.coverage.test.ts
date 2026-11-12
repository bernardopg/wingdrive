import {beforeEach, describe, expect, test} from 'bun:test';
import {useUndoStore} from './undoStore';

beforeEach(() => useUndoStore.setState({entries: [], busy: false}));

const record = (
	libraryId: string,
	label: string,
	run: () => Promise<void> = async () => {}
) => useUndoStore.getState().record({libraryId, label, run});

describe('undo history', () => {
	test('returns null for an empty history', async () => {
		expect(await useUndoStore.getState().undo('a')).toBeNull();
	});
	test('does not run entries belonging to another library', async () => {
		let called = false;
		record('b', 'other', async () => {
			called = true;
		});
		expect(await useUndoStore.getState().undo('a')).toBeNull();
		expect(called).toBe(false);
		expect(useUndoStore.getState().entries).toHaveLength(1);
	});
	test('undoes the most recent entry for the requested library', async () => {
		const calls: string[] = [];
		record('a', 'first', async () => {
			calls.push('first');
		});
		record('b', 'other', async () => {
			calls.push('other');
		});
		record('a', 'last', async () => {
			calls.push('last');
		});
		expect(await useUndoStore.getState().undo('a')).toBe('last');
		expect(calls).toEqual(['last']);
		expect(
			useUndoStore.getState().entries.map((entry) => entry.label)
		).toEqual(['first', 'other']);
	});
	test('successive undo calls walk backward', async () => {
		record('a', 'first');
		record('a', 'second');
		expect(await useUndoStore.getState().undo('a')).toBe('second');
		expect(await useUndoStore.getState().undo('a')).toBe('first');
		expect(await useUndoStore.getState().undo('a')).toBeNull();
	});
	test('assigns distinct ids even when labels are identical', () => {
		record('a', 'rename');
		record('a', 'rename');
		expect(
			new Set(useUndoStore.getState().entries.map((entry) => entry.id))
				.size
		).toBe(2);
	});
	test('retains the newest twenty entries', () => {
		for (let i = 0; i < 23; i++) record('a', String(i));
		expect(
			useUndoStore.getState().entries.map((entry) => entry.label)
		).toEqual(Array.from({length: 20}, (_, i) => String(i + 3)));
	});
	test('blocks concurrent undo until the operation finishes', async () => {
		let finish!: () => void;
		let calls = 0;
		record('a', 'pending', () => {
			calls++;
			return new Promise<void>((resolve) => {
				finish = resolve;
			});
		});
		const pending = useUndoStore.getState().undo('a');
		expect(useUndoStore.getState().busy).toBe(true);
		expect(await useUndoStore.getState().undo('a')).toBeNull();
		expect(calls).toBe(1);
		finish();
		expect(await pending).toBe('pending');
		expect(useUndoStore.getState().busy).toBe(false);
	});
	test('preserves a newer entry recorded while undo is pending', async () => {
		let finish!: () => void;
		record(
			'a',
			'pending',
			() =>
				new Promise<void>((resolve) => {
					finish = resolve;
				})
		);
		const pending = useUndoStore.getState().undo('a');
		record('a', 'new');
		finish();
		await pending;
		expect(
			useUndoStore.getState().entries.map((entry) => entry.label)
		).toEqual(['new']);
	});
	test('releases the busy flag after a rejection', async () => {
		record('a', 'failed', async () => {
			throw new Error('denied');
		});
		await expect(useUndoStore.getState().undo('a')).rejects.toThrow(
			'denied'
		);
		expect(useUndoStore.getState().busy).toBe(false);
		expect(
			useUndoStore.getState().entries.map((entry) => entry.label)
		).toEqual(['failed']);
	});
	test('can retry a failed operation', async () => {
		let attempts = 0;
		record('a', 'retry', async () => {
			if (++attempts === 1) throw new Error('temporary');
		});
		await expect(useUndoStore.getState().undo('a')).rejects.toThrow(
			'temporary'
		);
		expect(await useUndoStore.getState().undo('a')).toBe('retry');
		expect(attempts).toBe(2);
		expect(useUndoStore.getState().entries).toHaveLength(0);
	});
});
