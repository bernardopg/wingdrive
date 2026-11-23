import {describe, expect, it} from 'bun:test';

import {
	DEFAULT_EVENT_SUBSCRIPTION,
	NOISY_EVENTS,
	type EventVariant,
} from '../event-filter';

describe('event-filter', () => {
	it('excludes noisy events from the default subscription', () => {
		for (const event of NOISY_EVENTS) {
			expect(DEFAULT_EVENT_SUBSCRIPTION).not.toContain(event);
		}
	});

	it('includes core lifecycle and job events', () => {
		expect(DEFAULT_EVENT_SUBSCRIPTION).toContain('CoreStarted');
		expect(DEFAULT_EVENT_SUBSCRIPTION).toContain('CoreShutdown');
		expect(DEFAULT_EVENT_SUBSCRIPTION).toContain('JobProgress');
		expect(DEFAULT_EVENT_SUBSCRIPTION).toContain('JobCompleted');
		expect(DEFAULT_EVENT_SUBSCRIPTION).toContain('ResourceChangedBatch');
	});

	it('does not contain duplicates', () => {
		const unique = new Set(DEFAULT_EVENT_SUBSCRIPTION);
		expect(unique.size).toBe(DEFAULT_EVENT_SUBSCRIPTION.length);
	});

	it('marks only known noisy events as noisy', () => {
		expect(NOISY_EVENTS).toEqual(['IndexingProgress']);
	});

	it('keeps event variant typing usable at compile time', () => {
		const sample: EventVariant = 'JobStarted';
		expect(DEFAULT_EVENT_SUBSCRIPTION).toContain(sample);
	});
});
