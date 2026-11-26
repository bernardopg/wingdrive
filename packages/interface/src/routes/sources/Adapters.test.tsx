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
		translations: {pt: {summary: 'Busque no seu cofre do Obsidian.', description: null}},
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

let store: {sequence: number | null; generated_at: string | null; isolation: string; adapters: Card[]; updates: number};
const calls: Array<[string, unknown]> = [];
let fields: Array<Record<string, unknown>> = [];
let sourceConfig: {adapter_id: string; config: Record<string, unknown>; stored_secrets: string[]} | undefined;

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
		if (type === 'sources.config') return {data: sourceConfig, isLoading: false};
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
const {AdaptersScreen, ConfigureForm} = await import('./Adapters');

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
	sourceConfig = undefined;
	document.body.innerHTML = '';
	store = {sequence: 4, generated_at: null, isolation: 'bubblewrap', adapters: [card()], updates: 0};
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

	test('shows the summary in the interface language', async () => {
		await i18n.changeLanguage('pt');
		const host = document.createElement('div');
		document.body.appendChild(host);
		const root = createRoot(host);
		await act(async () => {
			root.render(
				<QueryClientProvider client={new QueryClient()}>
					<MemoryRouter initialEntries={['/sources/adapters']}>
						<AdaptersScreen />
					</MemoryRouter>
				</QueryClientProvider>
			);
		});
		expect(host.textContent).toContain('Busque no seu cofre do Obsidian.');
		act(() => root.unmount());
		await i18n.changeLanguage('en');
	});

	test('labels config fields in the interface language', async () => {
		store.adapters = [card({installed_version: '1.1.0'})];
		fields = [
			{
				key: 'vault_path',
				name: 'Vault',
				description: 'Your vault.',
				field_type: 'string',
				required: true,
				secret: false,
				default: null,
				path_type: 'folder',
				suggestions: [],
				translations: {pt: {name: 'Cofre', description: 'A pasta do seu cofre.'}}
			}
		];
		await i18n.changeLanguage('pt');
		const {root} = await render('/sources/adapters?adapter=obsidian');
		await i18n.changeLanguage('pt');
		await act(async () => button('Adicionar fonte')!.click());
		expect(document.body.textContent).toContain('Cofre');
		expect(document.body.textContent).toContain('A pasta do seu cofre.');
		expect(document.body.textContent).not.toContain('Your vault.');
		act(() => root.unmount());
		await i18n.changeLanguage('en');
	});

	test('says whether the permissions are enforced here', async () => {
		const enforced = await render('/sources/adapters?adapter=obsidian');
		expect(document.querySelector('[role="dialog"]')!.textContent).toContain('Enforced on this computer');
		act(() => enforced.root.unmount());
		document.body.innerHTML = '';
		store.isolation = 'none';
		const open = await render('/sources/adapters?adapter=obsidian');
		expect(document.querySelector('[role="dialog"]')!.textContent).toContain('Not enforced on this system');
		act(() => open.root.unmount());
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
			isolation: 'none',
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
				suggestions: ['/home/ana/Notes', '/home/ana/Work'],
				translations: {pt: {name: 'Cofre', description: 'A pasta do seu cofre.'}}
			},
			{
				key: 'include_starred',
				name: 'Starred',
				description: '',
				field_type: 'boolean',
				required: false,
				secret: false,
				default: 'true',
				path_type: null,
				suggestions: [],
				translations: {}
			}
		];
		const {root} = await render('/sources/adapters?adapter=obsidian');
		await act(async () => button('Add Source')!.click());
		const input = [...document.querySelectorAll('input')].find(i => i.value === '/home/ana/Notes');
		expect(input).toBeDefined();
		expect(document.body.textContent).toContain('Found 2 on this computer');
		const checkbox = document.querySelector<HTMLInputElement>('input[type=checkbox]')!;
		expect(checkbox.checked).toBe(true);
		await act(async () => checkbox.click());
		await act(async () => button('Add & Sync')!.click());
		expect(calls).toContainEqual([
			'sources.create',
			{name: 'Obsidian', adapter_id: 'obsidian', config: {vault_path: '/home/ana/Notes', include_starred: false}}
		]);
		expect(calls).toContainEqual(['sources.sync', {source_id: 'source-1'}]);
		act(() => root.unmount());
	});

	test('editing a source keeps a saved token unless a new one is typed', async () => {
		fields = [
			{
				key: 'username',
				name: 'Username',
				description: '',
				field_type: 'string',
				required: true,
				secret: false,
				default: null,
				path_type: null,
				suggestions: [],
				translations: {}
			},
			{
				key: 'token',
				name: 'Token',
				description: '',
				field_type: 'string',
				required: true,
				secret: true,
				default: null,
				path_type: null,
				suggestions: [],
				translations: {}
			},
			{
				key: 'label',
				name: 'Label',
				description: '',
				field_type: 'string',
				required: false,
				secret: false,
				default: null,
				path_type: null,
				suggestions: [],
				translations: {}
			}
		];
		sourceConfig = {adapter_id: 'github', config: {username: 'ana', label: 'old'}, stored_secrets: ['token']};
		await i18n.changeLanguage('en');
		const host = document.createElement('div');
		document.body.appendChild(host);
		const root = createRoot(host);
		await act(async () => {
			root.render(
				<QueryClientProvider client={new QueryClient()}>
					<MemoryRouter>
						<ConfigureForm
							adapterId="github"
							adapterName="GitHub"
							sourceId="source-9"
							sourceName="My GitHub"
							onDone={() => {}}
						/>
					</MemoryRouter>
				</QueryClientProvider>
			);
		});
		const token = document.querySelector<HTMLInputElement>('input[type=password]')!;
		expect(token.value).toBe('');
		expect(token.placeholder).toBe('Saved. Leave empty to keep it.');
		const label = [...document.querySelectorAll('input')].find(i => i.value === 'old')!;
		const setValue = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')!.set!;
		await act(async () => {
			setValue.call(label, '');
			label.dispatchEvent(new Event('input', {bubbles: true}));
		});
		await act(async () => button('Save & Sync')!.click());
		expect(calls).toContainEqual([
			'sources.update',
			{source_id: 'source-9', name: 'My GitHub', config: {username: 'ana', label: null}}
		]);
		expect(calls).toContainEqual(['sources.sync', {source_id: 'source-1'}]);
		act(() => root.unmount());
	});
});
