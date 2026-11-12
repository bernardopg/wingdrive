import {beforeEach, describe, expect, test} from 'bun:test';
import {useVolumeIndexingStore} from './volumeIndexingStore';

beforeEach(() => useVolumeIndexingStore.setState({volumeToJob: new Map()}));

describe('volume indexing jobs', () => {
	test('returns undefined for an unknown volume', () => {
		expect(
			useVolumeIndexingStore.getState().getJobId('missing')
		).toBeUndefined();
	});
	test('stores and retrieves a job', () => {
		useVolumeIndexingStore.getState().setVolumeJob('disk', 'job');
		expect(useVolumeIndexingStore.getState().getJobId('disk')).toBe('job');
	});
	test('replaces the job for a volume', () => {
		useVolumeIndexingStore.getState().setVolumeJob('disk', 'old');
		useVolumeIndexingStore.getState().setVolumeJob('disk', 'new');
		expect(useVolumeIndexingStore.getState().getJobId('disk')).toBe('new');
	});
	test('keeps jobs for other volumes', () => {
		useVolumeIndexingStore.getState().setVolumeJob('a', 'one');
		useVolumeIndexingStore.getState().setVolumeJob('b', 'two');
		expect(useVolumeIndexingStore.getState().getJobId('a')).toBe('one');
		expect(useVolumeIndexingStore.getState().getJobId('b')).toBe('two');
	});
	test('clears only the requested volume', () => {
		useVolumeIndexingStore.getState().setVolumeJob('a', 'one');
		useVolumeIndexingStore.getState().setVolumeJob('b', 'two');
		useVolumeIndexingStore.getState().clearVolumeJob('a');
		expect(useVolumeIndexingStore.getState().getJobId('a')).toBeUndefined();
		expect(useVolumeIndexingStore.getState().getJobId('b')).toBe('two');
	});
	test('clearing an unknown volume leaves known jobs intact', () => {
		useVolumeIndexingStore.getState().setVolumeJob('a', 'one');
		useVolumeIndexingStore.getState().clearVolumeJob('unknown');
		expect([...useVolumeIndexingStore.getState().volumeToJob]).toEqual([
			['a', 'one']
		]);
	});
	test('does not mutate the previous map when adding', () => {
		const previous = useVolumeIndexingStore.getState().volumeToJob;
		useVolumeIndexingStore.getState().setVolumeJob('a', 'one');
		expect(previous.size).toBe(0);
		expect(useVolumeIndexingStore.getState().volumeToJob).not.toBe(
			previous
		);
	});
	test('does not mutate the previous map when clearing', () => {
		useVolumeIndexingStore.getState().setVolumeJob('a', 'one');
		const previous = useVolumeIndexingStore.getState().volumeToJob;
		useVolumeIndexingStore.getState().clearVolumeJob('a');
		expect(previous.get('a')).toBe('one');
		expect(useVolumeIndexingStore.getState().volumeToJob).not.toBe(
			previous
		);
	});
});
