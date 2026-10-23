import './setup';
import {QueryClient, QueryObserver} from '@tanstack/react-query';
import {describe, expect, it} from 'bun:test';
import {scheduleCoalescedRefetch} from '../useNormalizedQuery';

function wait(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

describe('scheduleCoalescedRefetch', () => {
	it('refetches a listing once for a burst of change events', async () => {
		const queryClient = new QueryClient();
		const queryKey = ['query:files.directory_listing', 'library-id', {path: '/a'}];
		let fetches = 0;
		const observer = new QueryObserver(queryClient, {
			queryKey,
			queryFn: async () => {
				fetches += 1;
				await wait(30);
				return {files: [], fetch: fetches};
			}
		});
		const unsubscribe = observer.subscribe(() => {});
		await wait(50);
		expect(fetches).toBe(1);

		for (let i = 0; i < 200; i++) {
			scheduleCoalescedRefetch(queryClient, queryKey, 10);
		}
		expect(queryClient.getQueryState(queryKey)?.isInvalidated).toBe(true);
		await wait(100);

		expect(fetches).toBe(2);
		unsubscribe();
	});

	it('fetches again after a fetch that was running when the event arrived', async () => {
		const queryClient = new QueryClient();
		const queryKey = ['query:files.directory_listing', 'library-id', {path: '/b'}];
		let fetches = 0;
		const observer = new QueryObserver(queryClient, {
			queryKey,
			queryFn: async () => {
				fetches += 1;
				await wait(40);
				return {files: []};
			}
		});
		const unsubscribe = observer.subscribe(() => {});
		// The first fetch is still running when the event arrives.
		await wait(5);
		scheduleCoalescedRefetch(queryClient, queryKey, 10);
		await wait(150);

		expect(fetches).toBe(2);
		expect(queryClient.getQueryState(queryKey)?.isInvalidated).toBe(false);
		unsubscribe();
	});
});
