import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {beforeEach, describe, expect, mock, test} from 'bun:test';

if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();
(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

type Card = Record<string, unknown>;

function card(overrides: Card = {}): Card {
	return {
		id: 'obsidian',
		name: 'Obsidian',
		summary: 'Search your Obsidian vault.',
		description: 'Reads every Markdown note.',
		version: '1.1.0',
		author: 'WingDrive',
		license: 'Apache-2.0',
		homepage: null,
		categories: ['notes'],
		trust: 'official',
		permissions: {read: ['{config.vault_path}'], network: [], secrets: [], tools: []},
		platforms: ['linux', 'macos', 'windows'],
		supported_here: true,
		runtime_available: true,
		installed_version: null,
		update_available: false,
		update_needs_consent: false,
		revoked_reason: null,
		icon_svg: '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1" onload="alert(1)"/>',
		...overrides
	};
}

let store: {sequence: number | null; generated_at: string | null; adapters: Card[]; updates: number};
const calls: Array<[string, unknown]> = [];
let fields: Array<Record<string, unknown>> = [];

mock.module('../../contexts/WingDriveContext', () => ({
	useCoreQuery: ({type}: {type: string}) =>
		type === 'adapters.store.list' ? {data: store, isLoading: false} : {data: undefined},
	useLibraryQuery: ({type}: {type: string}) => {
		if (type === 'adapters.list') {
			return {
				data: store.adapters
					.filter(c => c.installed_version)
					.map(c => ({id: c.id, name: c.name, data_type: c.id, version: c.installed_version, icon_svg: null}))
			};
		}
		if (type === 'adapters.config') return {data: fields, isLoading: false, error: null};
		return {data: undefined};
	},
	useCoreMutation: (type: string) => ({
		isPending: false,
		error: null,
		mutateAsync: async (input: unknown) => {
			calls.push([type, input]);
			return {};
		}
	}),
	useLibraryMutation: (type: string) => ({
		isPending: false,
		mutateAsync: async (input: unknown) => {
			calls.push([type, input]);
			return {id: 'source-1'};
		}
	})
}));
const platformContext = await import('../../contexts/PlatformContext');
mock.module('../../contexts/PlatformContext', () => ({
	...platformContext,
	usePlatform: () => ({openDirectoryPickerDialog: async () => '/picked/Vault'})
}));
mock.module('../../TopBar', () => ({
	TopBarPortal: () => null,
	TopBarItem: ({children}: {children: unknown}) => children
}));

const {QueryClient, QueryClientProvider} = await import('@tanstack/react-query');
const {act} = await import('react');
const {createRoot} = await import('react-dom/client');
const {MemoryRouter} = await import('react-router-dom');
const {i18n} = await import('../../i18n');
const {AdaptersScreen} = await import('./Adapters');

async function render(path = '/sources/adapters') {
	await i18n.changeLanguage('en');
	const host = document.createElement('div');
	document.body.appendChild(host);
	const root = createRoot(host);
	await act(async () => {
		root.render(
			<QueryClientProvider client={new QueryClient()}>
				<MemoryRouter initialEntries={[path]}>
					<AdaptersScreen />
				</MemoryRouter>
			</QueryClientProvider>
		);
	});
	return {host, root};
}

function exact(text: string) {
	return [...document.querySelectorAll('button')].find(b => b.textContent === text);
}

function button(text: string) {
	return [...document.querySelectorAll('button')].find(b => b.textContent?.includes(text));
}

beforeEach(() => {
	calls.length = 0;
	fields = [];
	document.body.innerHTML = '';
	store = {sequence: 4, generated_at: null, adapters: [card()], updates: 0};
});

describe('adapter store', () => {
	test('shows a card with the summary, trust and an icon that cannot run script', async () => {
		const {host, root} = await render();
		expect(host.textContent).toContain('Obsidian');
		expect(host.textContent).toContain('Search your Obsidian vault.');
		expect(host.textContent).toContain('Official');
		expect(host.querySelector('[onload]')).toBeNull();
		expect(host.querySelector('img')?.getAttribute('src')).toStartWith('data:image/svg+xml');
		act(() => root.unmount());
	});

	test('lists permissions and installs with consent', async () => {
		const {root} = await render('/sources/adapters?adapter=obsidian');
		const dialog = document.querySelector('[role="dialog"]')!;
		expect(dialog.textContent).toContain('What it can access');
		expect(dialog.textContent).toContain('‹vault_path›');
		expect(dialog.textContent).toContain('nothing: it works offline');
		await act(async () => exact('Install')!.click());
		expect(calls).toContainEqual(['adapters.store.install', {id: 'obsidian', accept_permissions: true}]);
		act(() => root.unmount());
	});

	test('counts updates on the installed tab and warns about new permissions', async () => {
		store = {
			sequence: 5,
			generated_at: null,
			updates: 1,
			adapters: [card({installed_version: '1.0.0', update_available: true, update_needs_consent: true})]
		};
		const {host, root} = await render('/sources/adapters?adapter=obsidian');
		expect(button('Installed')!.textContent).toContain('1');
		expect(host.textContent).toContain('Update');
		const dialog = document.querySelector('[role="dialog"]')!;
		expect(dialog.textContent).toContain('asks for more access');
		await act(async () => button('Update to 1.1.0')!.click());
		expect(calls[0][0]).toBe('adapters.store.install');
		act(() => root.unmount());
	});

	test('a revoked version cannot become a source', async () => {
		store.adapters = [card({installed_version: '1.0.0', revoked_reason: 'leaks notes'})];
		const {root} = await render('/sources/adapters?adapter=obsidian');
		const dialog = document.querySelector('[role="dialog"]')!;
		expect(dialog.textContent).toContain('This version was disabled: leaks notes');
		expect(button('Add Source')).toBeUndefined();
		act(() => root.unmount());
	});

	test('adding a source pre-fills the detected value and syncs', async () => {
		store.adapters = [card({installed_version: '1.1.0'})];
		fields = [
			{
				key: 'vault_path',
				name: 'Vault',
				description: '',
				field_type: 'string',
				required: true,
				secret: false,
				default: null,
				path_type: 'folder',
				suggestions: ['/home/ana/Notes', '/home/ana/Work']
			}
		];
		const {root} = await render('/sources/adapters?adapter=obsidian');
		await act(async () => button('Add Source')!.click());
		const input = [...document.querySelectorAll('input')].find(i => i.value === '/home/ana/Notes');
		expect(input).toBeDefined();
		expect(document.body.textContent).toContain('Found 2 on this computer');
		await act(async () => button('Add & Sync')!.click());
		expect(calls).toContainEqual([
			'sources.create',
			{name: 'Obsidian', adapter_id: 'obsidian', config: {vault_path: '/home/ana/Notes'}}
		]);
		expect(calls).toContainEqual(['sources.sync', {source_id: 'source-1'}]);
		act(() => root.unmount());
	});
});
