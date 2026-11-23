import './setup';
import {describe, expect, it} from 'bun:test';
import {renderHook} from '@testing-library/react';
import {QueryClient, QueryClientProvider} from '@tanstack/react-query';
import type {ReactNode} from 'react';

import {WingDriveProvider, useWingDriveClient} from '../useClient';

function wrapper(client: any) {
	const qc = new QueryClient({defaultOptions: {queries: {retry: false}}});
	return ({children}: {children: ReactNode}) => (
		<QueryClientProvider client={qc}>
			<WingDriveProvider client={client}>{children}</WingDriveProvider>
		</QueryClientProvider>
	);
}

describe('useWingDriveClient', () => {
	it('returns the client from context', () => {
		const client = {getCurrentLibraryId: () => null} as any;
		const {result} = renderHook(() => useWingDriveClient(), {wrapper: wrapper(client)});
		expect(result.current).toBe(client);
	});

	it('throws when used outside provider', () => {
		expect(() => renderHook(() => useWingDriveClient())).toThrow(
			'useWingDriveClient must be used within WingDriveProvider',
		);
	});
});
