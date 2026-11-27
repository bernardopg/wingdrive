import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {describe, expect, it, mock} from 'bun:test';
import {QueryClient, QueryClientProvider} from '@tanstack/react-query';
import {act, type ReactElement} from 'react';
import {renderToStaticMarkup} from 'react-dom/server';
import type {Event} from '@wingdrive/ts-client';

if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();
(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

const accounts = [
	{
		id: '1001',
		provider: 'google',
		email: 'ana@example.com',
		name: 'Ana',
		picture: null,
		services: ['drive', 'photos'],
		mode: 'app',
		connected_at: '2026-11-12T10:00:00Z',
		drive: {fingerprint: 'fp', identifier: 'ana@example.com', mounted: true},
		drive_access: 'app_files',
		drive_picker: true
	},
	{
		id: '2002',
		provider: 'google',
		email: 'bruno@example.com',
		name: null,
		picture: null,
		services: ['photos'],
		mode: 'custom',
		connected_at: '2026-11-12T10:00:00Z',
		drive: null
	},
	{
		id: 'ms-carla',
		provider: 'microsoft',
		email: 'carla@outlook.com',
		name: 'Carla',
		picture: null,
		services: [],
		mode: 'app',
		connected_at: '2026-11-13T10:00:00Z',
		drive: {fingerprint: 'fp2', identifier: 'carla@outlook.com', mounted: true}
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
const bundledClients = {app: {source: 'bundled', client_id: 'abc.apps.googleusercontent.com'}, custom_client_id: null};
let clientInfo: {app: unknown; custom_client_id: string | null} = bundledClients;

const data: Record<string, unknown> = {
	'integrations.accounts.list': accounts,
	'cloud_sync.list': pairs,
	'files.cloud_edit.list': editSessions
};
const mutation = {mutateAsync: async () => ({}), mutate: () => {}, isPending: false};
const mutationOverrides = new Map<string, unknown>();
const openedUrls: string[] = [];
let subscribe: (callback: (event: Event) => void) => Promise<() => void> = async () => () => {};
const client = {getCurrentLibraryId: () => 'library', execute: async () => ({}), subscribe: (callback: (event: Event) => void) => subscribe(callback)};

// Bun module mocks leak into other test files, so keep the real exports and override only these
const actualContext = await import('../../contexts/WingDriveContext');
mock.module('../../contexts/WingDriveContext', () => ({
	...actualContext,
	useLibraryQuery: ({type}: {type: string}) => ({
		data: type === 'integrations.google.client' || type === 'integrations.microsoft.client' ? clientInfo : data[type],
		isLoading: false,
		refetch: async () => {}
	}),
	useLibraryMutation: (type: string) => mutationOverrides.get(type) ?? mutation,
	useWingDriveClient: () => client
}));
mock.module('../../contexts/PlatformContext', () => ({
	usePlatform: () => ({platform: 'web', openLink: (url: string) => openedUrls.push(url)})
}));

const {createRoot} = await import('react-dom/client');
const {ConnectPanel, IntegrationsSettings} = await import('./IntegrationsSettings');
const {i18n} = await import('../../i18n');

function render(element: ReactElement) {
	return renderToStaticMarkup(<QueryClientProvider client={new QueryClient()}>{element}</QueryClientProvider>);
}

describe('integrations settings', () => {
	it('refreshes accounts only for events in the current library and unsubscribes', async () => {
		let listener: ((event: Event) => void) | undefined;
		const unsubscribe = mock(() => {});
		subscribe = async callback => { listener = callback; return unsubscribe; };
		const queryClient = new QueryClient();
		const invalidate = mock(async () => {});
		queryClient.invalidateQueries = invalidate;
		const root = createRoot(document.createElement('div'));
		try {
			await act(async () => root.render(<QueryClientProvider client={queryClient}><IntegrationsSettings /></QueryClientProvider>));
			listener?.({Custom: {event_type: 'other'}});
			listener?.({IntegrationAccountChanged: {library_id: 'other', account_id: '1'}});
			expect(invalidate).not.toHaveBeenCalled();
			listener?.({IntegrationAccountChanged: {library_id: 'library', account_id: '1'}});
			expect(invalidate).toHaveBeenCalledWith({queryKey: ['integrations.accounts.list']});
			await act(async () => root.unmount());
			expect(unsubscribe).toHaveBeenCalledTimes(1);
		} finally {
			subscribe = async () => () => {};
		}
	});

	it('cleans up a subscription that resolves after unmount', async () => {
		let finish: ((unsubscribe: () => void) => void) | undefined;
		const unsubscribe = mock(() => {});
		subscribe = () => new Promise(resolve => { finish = resolve; });
		const root = createRoot(document.createElement('div'));
		try {
			await act(async () => root.render(<QueryClientProvider client={new QueryClient()}><IntegrationsSettings /></QueryClientProvider>));
			await act(async () => root.unmount());
			await act(async () => finish?.(unsubscribe));
			expect(unsubscribe).toHaveBeenCalledTimes(1);
		} finally {
			subscribe = async () => () => {};
		}
	});
	it('offers every service and connection mode as a card', () => {
		const html = render(<IntegrationsSettings />);
		expect(html).toContain('Connect a service');
		expect(html.match(/aria-pressed="false"/g)?.length).toBe(4);
		for (const modality of ['Google account', 'Google Cloud', 'Microsoft account', 'Microsoft Entra']) {
			expect(html).toContain(modality);
		}
		expect(html).toContain('Google Drive and Photos');
		expect(html).toContain('OneDrive');
		// WingDrive's clients are bundled; no client of the user's own is saved.
		expect(html).toContain('Ready');
		expect(html).toContain('Needs setup');
		// Ana signed in with WingDrive's client, Bruno with his own.
		expect(html).toContain('1 account');
		// Monochrome logos follow the text color.
		expect(html).toContain('fill="currentColor"');
	});

	it('lists Google and Microsoft accounts with how they signed in', () => {
		const html = render(<IntegrationsSettings />);
		expect(html).toContain('ana@example.com');
		expect(html).toContain('bruno@example.com');
		expect(html).toContain('carla@outlook.com');
		expect(html).toContain('gdrive://ana@example.com/');
		expect(html).toContain('onedrive://carla@outlook.com/');
		expect(html).toContain('Show OneDrive in WingDrive');
		expect(html).toContain('Import from Google Photos');
		// Bruno granted only Photos, so he is offered more access.
		expect(html).toContain('Grant Drive and Photos access');
	});

	it('offers the Google Picker only to drive.file accounts that can use it', () => {
		const html = render(<IntegrationsSettings />);
		expect(html).toContain('Add files from Google Drive');
		expect(html).toContain('Choose files...');
		expect(html).toContain('the ones you add below');
		expect(html).toContain('Set up full Drive access');
		// One button: Bruno has no Drive and Carla is on OneDrive.
		expect(html.match(/Choose files\.\.\./g)?.length).toBe(1);
		const ana = accounts[0] as {drive_picker: boolean};
		ana.drive_picker = false;
		try {
			const withoutKey = render(<IntegrationsSettings />);
			expect(withoutKey).not.toContain('Add files from Google Drive');
			expect(withoutKey).toContain('Drive shows only files WingDrive created or opened');
		} finally {
			ana.drive_picker = true;
		}
	});

	it('explains a mode with advantages, disadvantages and steps', () => {
		const html = render(
			<ConnectPanel entry={{provider: 'microsoft', mode: 'custom'}} clients={bundledClients as never} onSaved={() => {}} />
		);
		expect(html).toContain('OneDrive · Microsoft Entra');
		expect(html).toContain('Advantages');
		expect(html).toContain('Disadvantages');
		expect(html).toContain('How to connect');
		expect(html).toContain('http://localhost/oauth/callback');
		expect(html).toContain('Application (client) ID');
		// No app of the user's own is saved yet, so sign-in waits for one.
		expect(html).toMatch(/<button[^>]*disabled=""[^>]*>Sign in with Microsoft Entra/);
	});

	it('cancels sign-in when the settings panel closes before the provider responds', async () => {
		let finish: ((session: {session_id: string; auth_url: string}) => void) | undefined;
		const cancelled: string[] = [];
		mutationOverrides.set('integrations.google.connect', {
			...mutation, mutateAsync: () => new Promise((resolve) => { finish = resolve; })
		});
		mutationOverrides.set('integrations.google.connect_cancel', {
			...mutation, mutate: ({session_id}: {session_id: string}) => cancelled.push(session_id)
		});
		const host = document.createElement('div');
		const root = createRoot(host);
		try {
			await act(async () => root.render(
				<QueryClientProvider client={new QueryClient()}>
					<ConnectPanel entry={{provider: 'google', mode: 'app'}} clients={bundledClients as never} onSaved={() => {}} />
				</QueryClientProvider>
			));
			const connect = [...host.querySelectorAll('button')].find((button) => button.textContent?.trim() === 'Sign in with Google')!;
			await act(async () => connect.click());
			await act(async () => root.unmount());
			await act(async () => finish!({session_id: 'pending', auth_url: 'https://example.invalid/oauth'}));
			expect(cancelled).toEqual(['pending']);
			expect(openedUrls).toEqual([]);
		} finally {
			mutationOverrides.clear();
		}
	});

	it('warns when the build has no WingDrive client', () => {
		clientInfo = {app: null, custom_client_id: null};
		try {
			const html = render(<IntegrationsSettings />);
			expect(html).toContain('Not in this build');
			const panel = render(
				<ConnectPanel entry={{provider: 'google', mode: 'app'}} clients={clientInfo as never} onSaved={() => {}} />
			);
			expect(panel).toContain('does not include WingDrive');
			expect(panel).toMatch(/<button[^>]*disabled=""[^>]*>Sign in with Google/);
		} finally {
			clientInfo = bundledClients;
		}
	});

	it('shows sync pairs with their last result and open edits', () => {
		const html = render(<IntegrationsSettings />);
		expect(html).toContain('Documents');
		expect(html).toContain('ana@example.com/Backup/Docs');
		expect(html).toContain('3 up, 2 down, 1 conflicts');
		expect(html).toContain('notes (conflict copy).txt');
		expect(html).toContain('Finish editing');
	});

	it('renders in Portuguese', async () => {
		await i18n.changeLanguage('pt');
		try {
			const html = render(<IntegrationsSettings />);
			expect(html).toContain('Conectar um serviço');
			expect(html).toContain('Conta Google');
			expect(html).toContain('Contas conectadas');
			expect(html).toContain('Sincronização com a nuvem');
			expect(html).toContain('Adicionar arquivos do Google Drive');
			expect(html).toContain('Configurar acesso a todo o Drive');
			const panel = render(
				<ConnectPanel entry={{provider: 'google', mode: 'custom'}} clients={bundledClients as never} onSaved={() => {}} />
			);
			expect(panel).toContain('Vantagens');
			expect(panel).toContain('Desvantagens');
			expect(panel).toContain('Como conectar');
		} finally {
			await i18n.changeLanguage('en');
		}
	});
});
