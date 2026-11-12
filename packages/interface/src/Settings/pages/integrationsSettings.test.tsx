import {describe, expect, it, mock} from 'bun:test';
import {QueryClient, QueryClientProvider} from '@tanstack/react-query';
import type {ReactElement} from 'react';
import {renderToStaticMarkup} from 'react-dom/server';

const accounts = [
	{
		id: '1001',
		provider: 'google',
		email: 'ana@example.com',
		name: 'Ana',
		picture: null,
		services: ['drive', 'photos'],
		connected_at: '2026-11-12T10:00:00Z',
		drive: {fingerprint: 'fp', identifier: 'ana@example.com', mounted: true}
	},
	{
		id: '2002',
		provider: 'google',
		email: 'bruno@example.com',
		name: null,
		picture: null,
		services: ['photos'],
		connected_at: '2026-11-12T10:00:00Z',
		drive: null
	}
];
const pairs = [
	{
		id: 'p1',
		name: 'Documents',
		local_path: '/home/ana/Documents',
		remote: {Cloud: {service: 'gdrive', identifier: 'ana@example.com', path: 'Backup/Docs'}},
		mode: 'two_way',
		propagate_deletes: false,
		interval_minutes: 60,
		enabled: true,
		last_run_at: '2026-11-12T10:00:00Z',
		last_status: 'success',
		last_error: null,
		last_report: {uploaded: 3, downloaded: 2, deleted_local: 0, deleted_remote: 0, conflicts: 1, bytes_transferred: 10, errors: []},
		running: false
	}
];
const editSessions = [
	{
		id: 'e1',
		path: {Cloud: {service: 'gdrive', identifier: 'ana@example.com', path: 'notes.txt'}},
		local_path: '/tmp/e1/notes.txt',
		state: {state: 'conflict', copy_name: 'notes (conflict copy).txt'},
		last_synced_at: null
	}
];
let clientInfo = {configured: true, source: 'bundled', client_id: 'abc.apps.googleusercontent.com'};

const data: Record<string, unknown> = {
	'integrations.accounts.list': accounts,
	'cloud_sync.list': pairs,
	'files.cloud_edit.list': editSessions
};
const mutation = {mutateAsync: async () => ({}), mutate: () => {}, isPending: false};

// Bun module mocks leak into other test files, so keep the real exports and override only these
const actualContext = await import('../../contexts/WingDriveContext');
mock.module('../../contexts/WingDriveContext', () => ({
	...actualContext,
	useLibraryQuery: ({type}: {type: string}) => ({
		data: type === 'integrations.google.client' ? clientInfo : data[type],
		isLoading: false,
		refetch: async () => {}
	}),
	useLibraryMutation: () => mutation,
	useWingDriveClient: () => ({getCurrentLibraryId: () => 'library', execute: async () => ({})})
}));
mock.module('../../contexts/PlatformContext', () => ({
	usePlatform: () => ({platform: 'web', openLink: () => {}})
}));

const {IntegrationsSettings} = await import('./IntegrationsSettings');
const {i18n} = await import('../../i18n');

function render(element: ReactElement) {
	return renderToStaticMarkup(<QueryClientProvider client={new QueryClient()}>{element}</QueryClientProvider>);
}

describe('integrations settings', () => {
	it('lists every connected account with its services', () => {
		const html = render(<IntegrationsSettings />);
		expect(html).toContain('ana@example.com');
		expect(html).toContain('bruno@example.com');
		expect(html).toContain('gdrive://ana@example.com/');
		expect(html).toContain('Import from Google Photos');
		// Bruno granted only Photos, so he is offered more access.
		expect(html).toContain('Grant Drive and Photos access');
		expect(html).toContain('Connect a Google account');
	});

	it('shows sync pairs with their last result and open edits', () => {
		const html = render(<IntegrationsSettings />);
		expect(html).toContain('Documents');
		expect(html).toContain('ana@example.com/Backup/Docs');
		expect(html).toContain('3 up, 2 down, 1 conflicts');
		expect(html).toContain('notes (conflict copy).txt');
		expect(html).toContain('Finish editing');
	});

	it('explains the missing OAuth client and disables sign-in', () => {
		clientInfo = {configured: false, source: null as never, client_id: null as never};
		try {
			const html = render(<IntegrationsSettings />);
			expect(html).toContain('Sign-in needs a Google OAuth client');
			expect(html).toMatch(/<button[^>]*disabled=""[^>]*>Connect a Google account/);
		} finally {
			clientInfo = {configured: true, source: 'bundled', client_id: 'abc.apps.googleusercontent.com'};
		}
	});

	it('renders in Portuguese', async () => {
		await i18n.changeLanguage('pt');
		try {
			const html = render(<IntegrationsSettings />);
			expect(html).toContain('Contas Google');
			expect(html).toContain('Sincronização com a nuvem');
		} finally {
			await i18n.changeLanguage('en');
		}
	});
});
