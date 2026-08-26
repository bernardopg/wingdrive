import './setup';
import {QueryClient} from '@tanstack/react-query';
import {describe, expect, it} from 'bun:test';
import type {Event} from '../../generated/types';
import {handleResourceEvent} from '../useNormalizedQuery';

describe('useNormalizedQuery filtered collections', () => {
	it('invalidates the query when a matching resource changes', () => {
		const queryClient = new QueryClient();
		const queryKey = ['query:search.files', 'library-id', {favorite: true}];
		queryClient.setQueryData(queryKey, {files: [{id: 'file-id'}]});

		handleResourceEvent(
			{
				ResourceChanged: {
					resource_type: 'file',
					resource: {id: 'file-id', favorite: false},
					metadata: null
				}
			} as Event,
			{
				query: 'search.files',
				input: {favorite: true},
				resourceType: 'file',
				refetchOnResourceChange: true
			},
			queryKey,
			queryClient
		);

		expect(queryClient.getQueryState(queryKey)?.isInvalidated).toBe(true);
	});
});
