import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {afterEach, beforeEach, describe, expect, it, mock} from 'bun:test';
import type {ContextType} from 'react';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

type Listener = () => void;
type FilteredHandler = (event: unknown) => void;

let windowLabel: string | undefined = 'main';
let libraryId: string | null = 'lib-1';
const handlers: FilteredHandler[] = [];
let unsubscribed = 0;
const listeners = new Map<string, Set<Listener>>();
const client = {
	getCurrentLibraryId: () => libraryId,
	subscribeFiltered: async (_filter: unknown, handler: FilteredHandler) => {
		handlers.push(handler);
		return () => {
			unsubscribed += 1;
		};
	},
	on(event: string, listener: Listener) {
		if (!listeners.has(event)) listeners.set(event, new Set());
		listeners.get(event)!.add(listener);
	},
	off(event: string, listener: Listener) {
		listeners.get(event)?.delete(listener);
	}
};

// Bun module mocks leak into other test files, so keep the real exports.
const actualClient = await import('@wingdrive/ts-client');
mock.module('@wingdrive/ts-client', () => ({
	...actualClient,
	useWingDriveClient: () => client
}));
const actualPlatform = await import('../../contexts/PlatformContext');
mock.module('../../contexts/PlatformContext', () => ({
	...actualPlatform,
	usePlatform: () => ({getCurrentWindowLabel: () => windowLabel})
}));

const {act, useContext} = await import('react');
const {createRoot} = await import('react-dom/client');
const {TabManagerContext, TabManagerProvider, storageKeyFor} = await import(
	'./TabManagerContext'
);

(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

const routes = [{path: '*', element: null}];
type Manager = NonNullable<ContextType<typeof TabManagerContext>>;
let manager: Manager;
function Probe() {
	manager = useContext(TabManagerContext)!;
	return null;
}

let root: ReturnType<typeof createRoot> | undefined;
async function mount() {
	root = createRoot(document.createElement('div'));
	await act(async () => {
		root.render(
			<TabManagerProvider routes={routes}>
				<Probe />
			</TabManagerProvider>
		);
	});
}
const run = (fn: () => void) => act(async () => fn());
const titles = () => manager.tabs.map((tab) => tab.title);
const stored = (key = 'wing-tabs-state') => JSON.parse(localStorage.getItem(key) ?? 'null');

beforeEach(() => {
	localStorage.clear();
	windowLabel = 'main';
	libraryId = 'lib-1';
	handlers.length = 0;
	unsubscribed = 0;
	window.history.replaceState(null, '', '/');
});

afterEach(async () => {
	const mounted = root;
	root = undefined;
	if (mounted) await act(async () => mounted.unmount());
});

describe('storageKeyFor', () => {
	it('gives secondary windows their own entry and file choosers one shared entry', () => {
		expect(storageKeyFor()).toBe('wing-tabs-state');
		expect(storageKeyFor('main')).toBe('wing-tabs-state');
		expect(storageKeyFor('folder-2')).toBe('wing-tabs-state:folder-2');
		expect(storageKeyFor('file-chooser-7')).toBe('wing-tabs-state:file-chooser');
	});
});

describe('TabManagerProvider', () => {
	it('starts with one tab and persists what changes', async () => {
		await mount();
		expect(manager.tabs).toHaveLength(1);
		expect(manager.tabs[0]!.title).toBe('Overview');
		expect(manager.hasClosedTabs).toBe(false);
		expect(stored().activeTabId).toBe(manager.activeTabId);

		await run(() => manager.updateTabTitle(manager.activeTabId, 'Home'));
		await run(() => manager.updateTabPath(manager.activeTabId, '/explorer?path=/home'));
		expect(manager.tabs[0]).toMatchObject({title: 'Home', savedPath: '/explorer?path=/home'});
		expect(stored().tabs[0].title).toBe('Home');
	});

	it('creates tabs in front or in the background, and cycles through them', async () => {
		await mount();
		const first = manager.activeTabId;
		await run(() => manager.createTab('Two', '/two'));
		const second = manager.activeTabId;
		expect(second).not.toBe(first);
		await run(() => manager.createTab('Three', '/three', {background: true}));
		expect(manager.activeTabId).toBe(second);
		expect(titles()).toEqual(['Overview', 'Two', 'Three']);

		await run(() => manager.nextTab());
		expect(manager.tabs.find((t) => t.id === manager.activeTabId)!.title).toBe('Three');
		await run(() => manager.nextTab());
		expect(manager.activeTabId).toBe(first);
		await run(() => manager.previousTab());
		expect(manager.tabs.find((t) => t.id === manager.activeTabId)!.title).toBe('Three');
		await run(() => manager.selectTabAtIndex(1));
		expect(manager.activeTabId).toBe(second);
		await run(() => manager.selectTabAtIndex(9));
		expect(manager.activeTabId).toBe(second);
		await run(() => manager.switchTab(second));
		expect(manager.activeTabId).toBe(second);

		// A new tab without a title takes one from its path, or the default path.
		await run(() => manager.setDefaultNewTabPath('/recents'));
		await run(() => manager.createTab());
		expect(manager.tabs.at(-1)!.savedPath).toBe('/recents');
	});

	it('reorders tabs and ignores unknown or identical targets', async () => {
		await mount();
		const first = manager.activeTabId;
		await run(() => manager.createTab('Two', '/two'));
		const second = manager.activeTabId;
		await run(() => manager.reorderTabs(second, first));
		expect(titles()).toEqual(['Two', 'Overview']);
		await run(() => manager.reorderTabs(second, second));
		await run(() => manager.reorderTabs('missing', first));
		expect(titles()).toEqual(['Two', 'Overview']);
	});

	it('closes tabs, keeps one open, and reopens with the explorer state', async () => {
		await mount();
		const first = manager.activeTabId;
		await run(() => manager.createTab('Two', '/two'));
		const second = manager.activeTabId;
		await run(() => manager.createTab('Three', '/three'));
		const third = manager.activeTabId;
		await run(() => manager.updateExplorerState(second, {viewMode: 'list', scrollTop: 40}));

		// Closing the active tab falls back to the nearest one on the left.
		await run(() => manager.closeTab(third));
		expect(manager.activeTabId).toBe(second);
		await run(() => manager.closeTabs([first, second]));
		expect(manager.tabs).toHaveLength(2);
		await run(() => manager.closeTabs(['missing']));
		await run(() => manager.closeTab(first));
		expect(manager.activeTabId).toBe(second);
		expect(manager.hasClosedTabs).toBe(true);

		await run(() => manager.closeTab(second));
		expect(manager.tabs.map((t) => t.id)).toEqual([second]);

		await run(() => manager.reopenTab());
		expect(manager.activeTabId).toBe(first);
		await run(() => manager.reopenTab());
		expect(manager.activeTabId).toBe(third);
		expect(manager.hasClosedTabs).toBe(false);
		await run(() => manager.reopenTab());
		expect(manager.tabs).toHaveLength(3);
		expect(manager.getExplorerState(second)).toMatchObject({viewMode: 'list', scrollTop: 40});
	});

	it('keeps explorer and selection state per tab and skips no-op updates', async () => {
		await mount();
		const tab = manager.activeTabId;
		const before = manager.getExplorerState(tab);
		await run(() => manager.updateExplorerState(tab, {viewMode: 'grid'}));
		expect(manager.getExplorerState(tab)).toBe(before);
		await run(() => manager.updateExplorerState(tab, {showHiddenFiles: true}));
		expect(manager.getExplorerState(tab).showHiddenFiles).toBe(true);
		expect(manager.getExplorerState('unknown').viewMode).toBe('grid');

		await run(() => manager.updateSelectionIds(tab, ['a', 'b']));
		const ids = manager.getSelectionIds(tab);
		await run(() => manager.updateSelectionIds(tab, ['a', 'b']));
		expect(manager.getSelectionIds(tab)).toBe(ids);
		expect(manager.getSelectionIds('unknown')).toEqual([]);
	});

	it('remaps selected ids when the core reconciles a file and clears them on library change', async () => {
		await mount();
		const tab = manager.activeTabId;
		await run(() => manager.updateSelectionIds(tab, ['old-id', 'other']));
		expect(handlers).toHaveLength(1);
		await run(() => {
			handlers[0]!({
				ResourceChanged: {
					resource_type: 'file',
					resource: {id: 'new-id'},
					metadata: {alternate_ids: ['old-id']}
				}
			});
			// Events that are not file reconciliations are ignored.
			handlers[0]!({Other: {}});
			handlers[0]!({ResourceChanged: {resource_type: 'tag', resource: {id: 'x'}}});
		});
		expect(manager.getSelectionIds(tab)).toEqual(['new-id', 'other']);

		await run(() => {
			for (const listener of listeners.get('library-changed') ?? []) listener();
		});
		expect(manager.getSelectionIds(tab)).toEqual([]);
		expect(handlers).toHaveLength(2);
		expect(unsubscribed).toBe(1);
	});

	it('opens a split pane in the same folder, toggles it and closes it with its tab', async () => {
		await mount();
		await run(() => manager.updateTabPath(manager.activeTabId, '/explorer?path=/srv'));
		expect(manager.split).toBeNull();
		expect(manager.panePath('secondary')).toBeNull();
		await run(() => manager.toggleSplit());
		expect(manager.split).not.toBeNull();
		expect(manager.focusedPane).toBe('secondary');
		expect(manager.panePath('secondary')).toBe('/explorer?path=/srv');
		expect(manager.panePath('primary')).toBe('/explorer?path=/srv');

		await run(() => manager.navigatePane('secondary', '/recents'));
		await run(() => manager.navigatePane('primary', '/favorites'));
		await run(() => manager.setFocusedPane('primary'));
		expect(manager.focusedPane).toBe('primary');

		await run(() => manager.toggleSplit());
		expect(manager.split).toBeNull();
		expect(manager.focusedPane).toBe('primary');
		await run(() => manager.toggleSplit());
		expect(manager.split).not.toBeNull();
		expect(Object.keys(stored().splits)).toEqual([manager.activeTabId]);

		const owner = manager.activeTabId;
		await run(() => manager.createTab('Two', '/two'));
		await run(() => manager.closeTab(owner));
		expect(stored().splits).toEqual({});
	});

	it('restores valid saved tabs and rejects a corrupt or remote entry', async () => {
		localStorage.setItem(
			'wing-tabs-state',
			JSON.stringify({
				tabs: [
					{id: 't1', title: 'One', icon: null, isPinned: false, lastActive: 1, savedPath: '/one'},
					{
						id: 't2',
						title: 'Two',
						icon: null,
						isPinned: false,
						lastActive: 2,
						savedPath: '/two',
						history: ['/a', '/two'],
						historyIndex: 1
					}
				],
				activeTabId: 't2',
				explorerStates: {t2: {viewMode: 'list'}},
				defaultNewTabPath: 'relative',
				splits: {
					t2: {tab: {id: 's2', title: 'Two', savedPath: '/two'}, open: true},
					gone: {tab: {id: 's3', title: 'X', savedPath: '/x'}, open: true}
				}
			})
		);
		await mount();
		expect(titles()).toEqual(['One', 'Two']);
		expect(manager.activeTabId).toBe('t2');
		expect(manager.getExplorerState('t2').viewMode).toBe('list');
		expect(manager.split?.tabId).toBe('s2');
		await run(() => manager.createTab());
		// A relative default path is replaced with the root.
		expect(manager.tabs.at(-1)!.savedPath).toBe('/');
		await act(async () => root!.unmount());

		for (const corrupt of [
			'{not json',
			JSON.stringify({tabs: [], activeTabId: 'x', explorerStates: {}}),
			JSON.stringify({
				tabs: [{id: 'a', title: 'A', savedPath: '//server/share'}],
				activeTabId: 'a',
				explorerStates: {}
			}),
			JSON.stringify({
				tabs: [
					{id: 'a', title: 'A', savedPath: '/a'},
					{id: 'a', title: 'B', savedPath: '/b'}
				],
				activeTabId: 'a',
				explorerStates: {}
			})
		]) {
			localStorage.setItem('wing-tabs-state', corrupt);
			await mount();
			expect(titles()).toEqual(['Overview']);
			await act(async () => root!.unmount());
		}
		await mount();
	});

	it('a secondary window keeps its tabs under its own key', async () => {
		windowLabel = 'folder-3';
		await mount();
		await run(() => manager.updateTabTitle(manager.activeTabId, 'Elsewhere'));
		expect(stored('wing-tabs-state:folder-3').tabs[0].title).toBe('Elsewhere');
		expect(localStorage.getItem('wing-tabs-state')).toBeNull();
	});

	it('without a library nothing subscribes', async () => {
		libraryId = null;
		await mount();
		expect(handlers).toHaveLength(0);
	});
});
