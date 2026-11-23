import './setup';
import {describe, expect, it} from 'bun:test';
import {renderHook, waitFor} from '@testing-library/react';
import {QueryClient, QueryClientProvider} from '@tanstack/react-query';
import type {ReactNode} from 'react';

import {WingDriveProvider} from '../useClient';
import {useCoreMutation, useLibraryMutation} from '../useMutation';

function makeWrapper(client: any) {
	const qc = new QueryClient({defaultOptions: {queries: {retry: false}}});
	return ({children}: {children: ReactNode}) => (
		<QueryClientProvider client={qc}>
			<WingDriveProvider client={client}>{children}</WingDriveProvider>
		</QueryClientProvider>
	);
}

describe('useMutation hooks', () => {
	it('useCoreMutation executes wire method', async () => {
		const calls: any[] = [];
		const client = {
			getCurrentLibraryId: () => null,
			execute: async (method: string, input: any) => {
				calls.push({method, input});
				return {id: 'lib-1'};
			},
		};
		const {result} = renderHook(
			() => useCoreMutation('libraries.create' as any),
			{wrapper: makeWrapper(client)},
		);

		result.current.mutate({name: 'Test'} as any);
		await waitFor(() => expect(result.current.isSuccess).toBe(true));
		expect(calls[0]).toEqual({
			method: 'action:libraries.create.input',
			input: {name: 'Test'},
		});
	});

	it('useLibraryMutation throws when no library is selected', async () => {
		const client = {
			getCurrentLibraryId: () => null,
			execute: async () => ({ok: true}),
		};
		const {result} = renderHook(
			() => useLibraryMutation('tags.apply' as any),
			{wrapper: makeWrapper(client)},
		);

		result.current.mutate({entry_ids: [1], tag_ids: ['t']} as any);
		await waitFor(() => expect(result.current.isError).toBe(true));
		expect((result.current.error as Error).message).toContain('No library selected');
	});

	it('useLibraryMutation executes when library is selected', async () => {
		const calls: any[] = [];
		const client = {
			getCurrentLibraryId: () => 'lib-1',
			execute: async (method: string, input: any) => {
				calls.push({method, input});
				return {ok: true};
			},
		};
		const {result} = renderHook(
			() => useLibraryMutation('tags.apply' as any),
			{wrapper: makeWrapper(client)},
		);

		result.current.mutate({entry_ids: [1], tag_ids: ['t']} as any);
		await waitFor(() => expect(result.current.isSuccess).toBe(true));
		expect(calls[0].method).toBe('action:tags.apply.input');
	});
});
