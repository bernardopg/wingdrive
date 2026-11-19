import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {afterEach, beforeEach, describe, expect, it, mock} from 'bun:test';
import type {ReactNode} from 'react';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

const React = await import('react');
const {act, createContext, useCallback, useContext, useMemo, useState} = React;

interface FakeTabs {
	activeTabId: string;
	tabs: Array<{id: string; history?: string[]; historyIndex?: number}>;
	getExplorerState: (id: string) => Record<string, unknown>;
	updateExplorerState: (id: string, updates: Record<string, unknown>) => void;
}
const FakeTabsContext = createContext<FakeTabs | null>(null);
let history = {history: ['/a'], historyIndex: 0};
const baseState = {
	viewMode: 'grid',
	sortBy: 'name',
	sortDirection: null,
	gridSize: 120,
	gapSize: 16,
	foldersFirst: true,
	showHiddenFiles: false,
	groupItems: false,
	columnStack: [] as string[],
	scrollTop: 0,
	scrollLeft: 0,
	sizeViewTransform: {k: 1, x: 0, y: 0}
};
function FakeTabsProvider({children}: {children: ReactNode}) {
	const [states, setStates] = useState<Record<string, Record<string, unknown>>>({});
	const getExplorerState = useCallback(
		(id: string) => states[id] ?? baseState,
		[states]
	);
	const updateExplorerState = useCallback((id: string, updates: Record<string, unknown>) => {
		setStates((prev) => ({...prev, [id]: {...(prev[id] ?? baseState), ...updates}}));
	}, []);
	const value = useMemo(
		() => ({
			activeTabId: 'tab-1',
			tabs: [{id: 'tab-1', ...history}],
			getExplorerState,
			updateExplorerState
		}),
		[getExplorerState, updateExplorerState]
	);
	return <FakeTabsContext.Provider value={value}>{children}</FakeTabsContext.Provider>;
}

const viewPrefs = {
	saved: new Map<string, unknown>(),
	getPreferences(key: string) {
		return this.saved.get(key) as never;
	},
	setPreferences(key: string, prefs: unknown) {
		this.saved.set(key, {...(this.saved.get(key) as object), ...(prefs as object)});
	}
};
const sortPrefs = {
	saved: new Map<string, string>(),
	getPreferences(key: string) {
		return this.saved.get(key);
	},
	setPreferences(key: string, sort: string) {
		this.saved.set(key, sort);
	}
};
let devices: Array<{id: string; name: string}> | undefined = [{id: 'd1', name: 'Laptop'}];

// Bun module mocks leak into other test files, so keep the real exports.
const actualClient = await import('@wingdrive/ts-client');
mock.module('@wingdrive/ts-client', () => ({
	...actualClient,
	useViewPreferencesStore: () => viewPrefs,
	useSortPreferencesStore: () => sortPrefs
}));
const actualTabs = await import('../../components/TabManager/useTabManager');
mock.module('../../components/TabManager/useTabManager', () => ({
	...actualTabs,
	useTabManager: () => useContext(FakeTabsContext)!
}));
const actualWingDrive = await import('../../contexts/WingDriveContext');
mock.module('../../contexts/WingDriveContext', () => ({
	...actualWingDrive,
	useNormalizedQuery: () => ({data: devices})
}));

const {createRoot} = await import('react-dom/client');
const {MemoryRouter, useLocation} = await import('react-router-dom');
const {SelectionContext} = await import('./SelectionContext');
const explorer = await import('./context');
const {ExplorerProvider, useExplorer, useOptionalExplorer} = explorer;

(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

let ctx: ReturnType<typeof useExplorer>;
let location: ReturnType<typeof useLocation>;
const restored: unknown[][] = [];
function Probe() {
	ctx = useExplorer();
	location = useLocation();
	return null;
}

const home = {Physical: {device_slug: 'local', path: '/home/me'}};
const docs = {Physical: {device_slug: 'local', path: '/home/me/docs'}};
const pathUrl = (path: unknown) => explorer.targetToUrl({type: 'path', path: path as never});

let root: ReturnType<typeof createRoot> | undefined;
async function mount(entry = pathUrl(home)) {
	root = createRoot(document.createElement('div'));
	const selection = {restoreSelectionFromFiles: (files: unknown[]) => restored.push(files)};
	await act(async () => {
		root!.render(
			<MemoryRouter initialEntries={['/', entry]} initialIndex={1}>
				<FakeTabsProvider>
					<SelectionContext.Provider value={selection as never}>
						<ExplorerProvider>
							<Probe />
						</ExplorerProvider>
					</SelectionContext.Provider>
				</FakeTabsProvider>
			</MemoryRouter>
		);
	});
}
const run = (fn: () => void) => act(async () => fn());
const file = (id: string) => ({id}) as never;

beforeEach(() => {
	viewPrefs.saved.clear();
	sortPrefs.saved.clear();
	restored.length = 0;
	devices = [{id: 'd1', name: 'Laptop'}];
	history = {history: ['/a'], historyIndex: 0};
});

afterEach(async () => {
	const mounted = root;
	root = undefined;
	if (mounted) await act(async () => mounted.unmount());
});

describe('explorer helpers', () => {
	it('sorts names ascending and dates or sizes descending by default', () => {
		expect(explorer.effectiveSortDirection('name', null)).toBe('Asc');
		expect(explorer.effectiveSortDirection('type', undefined)).toBe('Asc');
		expect(explorer.effectiveSortDirection('modified', null)).toBe('Desc');
		expect(explorer.effectiveSortDirection('size', 'Asc')).toBe('Asc');
	});

	it('builds urls and keys for paths and views', () => {
		expect(pathUrl(home)).toBe(
			`/explorer?path=${encodeURIComponent(JSON.stringify(home))}`
		);
		expect(
			explorer.targetToUrl({type: 'view', view: 'device', id: 'd1', params: {tab: 'x'}})
		).toBe('/explorer?view=device&id=d1&tab=x');
		expect(explorer.targetToKey({type: 'path', path: home as never})).toBe(
			'path:local:/home/me'
		);
		expect(explorer.targetToKey({type: 'path', path: {Virtual: 'trash'} as never})).toBe(
			'path:virtual:trash'
		);
		expect(explorer.targetToKey({type: 'path', path: {Cloud: {}} as never})).toBe(
			'path:{"Cloud":{}}'
		);
		expect(explorer.targetToKey({type: 'view', view: 'recents'})).toBe('view:recents:');
		expect(explorer.targetsEqual(null, null)).toBe(true);
		expect(explorer.targetsEqual(null, {type: 'view', view: 'x'})).toBe(false);
		expect(
			explorer.targetsEqual(
				{type: 'path', path: home as never},
				{type: 'path', path: {...home} as never}
			)
		).toBe(true);
	});

	it('maps routes to the sidebar item keys their preferences are saved under', () => {
		expect(explorer.getWingItemKey('/', '')).toBe('overview');
		expect(explorer.getWingItemKey('/recents', '')).toBe('recents');
		expect(explorer.getWingItemKey('/favorites', '')).toBe('favorites');
		expect(explorer.getWingItemKey('/file-kinds', '')).toBe('file-kinds');
		expect(explorer.getWingItemKey('/tag/t1', '')).toBe('tag:t1');
		expect(explorer.getWingItemKey('/explorer', '?view=x')).toBe('explorer:?view=x');
		expect(explorer.getWingItemKey('/settings', '')).toBe('/settings');
	});

	it('compares file lists by identity and order', () => {
		const a = file('a');
		const b = file('b');
		const list = [a, b];
		expect(explorer.sameFiles(list, list)).toBe(true);
		expect(explorer.sameFiles([a, b], [a, b])).toBe(true);
		expect(explorer.sameFiles([a, b], [b, a])).toBe(false);
		expect(explorer.sameFiles([a], [a, b])).toBe(false);
	});
});

describe('ExplorerProvider', () => {
	it('reads the path from the url and navigates to paths and views', async () => {
		await mount();
		expect(ctx.currentPath).toEqual(home);
		expect(ctx.currentView).toBeNull();
		expect(ctx.operationalPath).toEqual(home);
		expect(ctx.devices.get('d1')?.name).toBe('Laptop');
		expect(ctx.activeTabId).toBe('tab-1');

		await run(() => ctx.navigateToPath(docs as never));
		expect(ctx.currentPath).toEqual(docs);
		await run(() => ctx.navigateToView('device', 'd1', {from: 'sidebar'}));
		expect(ctx.currentView).toEqual({view: 'device', id: 'd1', params: {from: 'sidebar'}});
		expect(ctx.currentPath).toBeNull();
		expect(ctx.operationalPath).toBeNull();
	});

	it('goes back and forward only when the tab history allows it', async () => {
		history = {history: ['/', '/b', '/c'], historyIndex: 1};
		await mount();
		expect(ctx.canGoBack).toBe(true);
		expect(ctx.canGoForward).toBe(true);
		await run(() => ctx.goBack());
		expect(location.pathname).toBe('/');
		await run(() => ctx.goForward());
		expect(location.pathname).toBe('/explorer');
		await act(async () => root!.unmount());

		history = {history: ['/'], historyIndex: 0};
		await mount();
		expect(ctx.canGoBack).toBe(false);
		await run(() => ctx.goBack());
		await run(() => ctx.goForward());
		expect(location.pathname).toBe('/explorer');
	});

	it('an invalid path parameter yields no target', async () => {
		await mount('/explorer?path=not-json');
		expect(ctx.currentTarget).toBeNull();
		await act(async () => root!.unmount());
		await mount('/explorer');
		expect(ctx.currentTarget).toBeNull();
	});

	it('publishes the files of the current view only and restores the selection from them', async () => {
		await mount();
		const files = [file('a'), file('b')];
		await run(() => ctx.setCurrentFiles(files, 5));
		expect(ctx.currentFiles).toBe(files);
		expect(ctx.listedFiles).toBe(files);
		expect(ctx.unfilteredFileCount).toBe(5);
		expect(restored.at(-1)).toBe(files);

		const before = ctx.currentFiles;
		await run(() => ctx.setCurrentFiles([...files], 5));
		expect(ctx.currentFiles).toBe(before);

		const listed = [files[0]!];
		await run(() => ctx.setCurrentFiles(files, undefined, listed));
		expect(ctx.listedFiles).toBe(listed);
		expect(ctx.unfilteredFileCount).toBeNull();

		// A navigation drops what the previous folder published.
		await run(() => ctx.navigateToPath(docs as never));
		expect(ctx.currentFiles).toEqual([]);
	});

	it('keeps a folder filter for the folder it was typed in', async () => {
		await mount();
		await run(() => ctx.setFolderFilter('rep'));
		expect(ctx.folderFilter).toBe('rep');
		await run(() => ctx.navigateToPath(docs as never));
		expect(ctx.folderFilter).toBeNull();
	});

	it('uses the active column as the operational folder in column view', async () => {
		await mount();
		await run(() => ctx.setViewMode('column'));
		expect(ctx.viewMode).toBe('column');
		expect(ctx.operationalPath).toBeNull();
		await run(() => ctx.setActiveColumnPath(docs as never));
		expect(ctx.operationalPath).toEqual(docs);
		await run(() => ctx.setColumnStack([home, docs] as never));
		expect(ctx.columnStack).toEqual([home, docs]);
		expect(viewPrefs.saved.get(`explorer:${location.search}`)).toEqual({viewMode: 'column'});
	});

	it('flips the direction when the active sort is picked again and saves it per path', async () => {
		await mount();
		expect(ctx.sortBy).toBe('name');
		expect(ctx.sortDirection).toBe('Asc');
		await run(() => ctx.setSortBy('name'));
		expect(ctx.sortDirection).toBe('Desc');
		await run(() => ctx.setSortBy('size'));
		expect(ctx.sortBy).toBe('size');
		expect(ctx.sortDirection).toBe('Desc');
		expect(sortPrefs.saved.get('path:local:/home/me')).toBe('size');
	});

	it('swaps sorts that only fit media view', async () => {
		sortPrefs.saved.set('path:local:/home/me', 'datetaken');
		await mount();
		expect(sortPrefs.saved.get('path:local:/home/me')).toBe('modified');
		await act(async () => root!.unmount());

		sortPrefs.saved.set('path:local:/home/me', 'type');
		await mount();
		await run(() => ctx.loadPreferencesForWingItem('none'));
		viewPrefs.saved.set('media-item', {viewMode: 'media', viewSettings: {gridSize: 200}});
		await run(() => ctx.loadPreferencesForWingItem('media-item'));
		expect(sortPrefs.saved.get('path:local:/home/me')).toBe('datetaken');
	});

	it('splits view settings between the tab and the global ui state', async () => {
		await mount();
		await run(() => ctx.setViewSettings({gridSize: 180, showHiddenFiles: true}));
		expect(ctx.viewSettings.gridSize).toBe(180);
		expect(ctx.viewSettings.showHiddenFiles).toBe(true);
		await run(() =>
			ctx.setViewSettings({showFileSize: false, calculateFolderSizes: true, sizeViewItemLimit: 50})
		);
		expect(ctx.viewSettings).toMatchObject({
			showFileSize: false,
			calculateFolderSizes: true,
			sizeViewItemLimit: 50,
			gridSize: 180
		});

		await run(() => ctx.setScrollPosition({top: 30, left: 4}));
		expect(ctx.scrollPosition).toEqual({top: 30, left: 4});
		await run(() => ctx.setSizeViewTransform({k: 2, x: 1, y: 1}));
		expect(ctx.sizeViewTransform).toEqual({k: 2, x: 1, y: 1});
	});

	it('switches between search, recents, tag and filtered modes', async () => {
		await mount();
		await run(() => ctx.enterSearchMode('report', 'library'));
		expect(ctx.mode).toEqual({type: 'search', query: 'report', scope: 'library'});
		expect(ctx.operationalPath).toBeNull();
		// Typing refines the query and keeps the scope.
		await run(() => ctx.enterSearchMode('reports'));
		expect(ctx.mode).toEqual({type: 'search', query: 'reports', scope: 'library'});
		await run(() => ctx.setSearchFilters({size: 'huge'}));
		await run(() => ctx.setSearchPage(-3));
		expect(ctx.searchPage).toBe(0);
		await run(() => ctx.setSearchPage(2));
		expect(ctx.searchPage).toBe(2);
		await run(() => ctx.exitSearchMode());
		expect(ctx.mode).toEqual({type: 'browse'});
		expect(ctx.searchFilters).toEqual({});

		await run(() => ctx.enterSearchMode('x'));
		expect(ctx.mode).toMatchObject({scope: 'folder'});
		await run(() => ctx.enterRecentsMode());
		expect(ctx.mode.type).toBe('recents');
		await run(() => ctx.exitRecentsMode());
		await run(() => ctx.enterTagMode('t1'));
		expect(ctx.mode).toEqual({type: 'tag', tagId: 't1'});
		await run(() => ctx.exitTagMode());
		await run(() => ctx.enterFilteredMode({} as never, 'Images'));
		expect(ctx.mode).toMatchObject({type: 'filtered', label: 'Images'});
		await run(() => ctx.exitFilteredMode());
		expect(ctx.mode.type).toBe('browse');
	});

	it('toggles panels, quick preview and tag mode', async () => {
		devices = undefined;
		await mount();
		expect(ctx.devices.size).toBe(0);
		await run(() => ctx.setSidebarVisible(false));
		await run(() => ctx.setInspectorVisible(false));
		await run(() => ctx.openQuickPreview('f1'));
		await run(() => ctx.setTagModeActive(true));
		expect(ctx).toMatchObject({
			sidebarVisible: false,
			inspectorVisible: false,
			quickPreviewFileId: 'f1',
			tagModeActive: true
		});
		await run(() => ctx.closeQuickPreview());
		expect(ctx.quickPreviewFileId).toBeNull();
	});

	it('closes quick preview when navigating to another folder', async () => {
		await mount();
		await run(() => ctx.openQuickPreview('f1'));
		await run(() => ctx.navigateToPath(docs));
		expect(ctx.quickPreviewFileId).toBeNull();
	});

	it('useExplorer outside the provider throws; the optional variant returns null', async () => {
		let optional: unknown = 'unset';
		function Outside() {
			optional = useOptionalExplorer();
			return null;
		}
		root = createRoot(document.createElement('div'));
		await act(async () => root!.render(<Outside />));
		expect(optional).toBeNull();

		function Strict() {
			useExplorer();
			return null;
		}
		const errors = console.error;
		console.error = () => {};
		try {
			let thrown: unknown;
			try {
				await act(async () => root!.render(<Strict />));
			} catch (error) {
				thrown = error;
			}
			expect((thrown as Error).message).toBe('useExplorer must be used within an ExplorerProvider');
		} finally {
			console.error = errors;
		}
	});
});
