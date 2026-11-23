import './setup';
import {describe, expect, it} from 'bun:test';
import {renderHook, waitFor} from '@testing-library/react';
import {QueryClient, QueryClientProvider} from '@tanstack/react-query';
import type {ReactNode} from 'react';

import {WingDriveProvider} from '../useClient';
import {useCoreQuery, useLibraryQuery} from '../useQuery';

function makeWrapper(client: any) {
	const qc = new QueryClient({
		defaultOptions: {queries: {retry: false, gcTime: 0}},
	});
	return ({children}: {children: ReactNode}) => (
		<QueryClientProvider client={qc}>
			<WingDriveProvider client={client}>{children}</WingDriveProvider>
		</QueryClientProvider>
	);
}

describe('useQuery hooks', () => {
	it('useCoreQuery executes wire method with input', async () => {
		const calls: any[] = [];
		const client = {
			getCurrentLibraryId: () => null,
			execute: async (method: string, input: any) => {
				calls.push({method, input});
				return {ok: true};
			},
		};
		const {result} = renderHook(
			() => useCoreQuery({type: 'libraries.list' as any, input: {} as any}),
			{wrapper: makeWrapper(client)},
		);

		await waitFor(() => expect(result.current.isSuccess).toBe(true));
		expect(calls[0].method).toBe('query:libraries.list');
		expect(result.current.data).toEqual({ok: true});
	});

	it('useLibraryQuery is disabled without a library and runs with one', async () => {
		const calls: any[] = [];
		const client = {
			current: null as string | null,
			getCurrentLibraryId() {
				return this.current;
			},
			execute: async (method: string, input: any) => {
				calls.push({method, input});
				return {entries: []};
			},
		};

		const {result, rerender} = renderHook(
			() => useLibraryQuery({type: 'files.directory_listing' as any, input: {path: '/'} as any}),
			{wrapper: makeWrapper(client)},
		);

		expect(result.current.fetchStatus).toBe('idle');
		expect(calls).toHaveLength(0);

		client.current = 'lib-1';
		rerender();
		await waitFor(() => expect(result.current.isSuccess).toBe(true));
		expect(calls[0].method).toBe('query:files.directory_listing');
	});
});
