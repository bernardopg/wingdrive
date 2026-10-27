import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {describe, expect, it, mock} from 'bun:test';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

const explorerState: {listSorting?: {id: string; desc: boolean}[]} = {};

// Bun module mocks leak into other test files, so keep the real exports
const actualContext = await import('../../context');
mock.module('../../context', () => ({
	...actualContext,
	useExplorer: () => ({
		sortBy: 'name',
		sortDirection: null,
		viewSettings: {foldersFirst: true}
	})
}));
const actualTabManager = await import('../../../../components/TabManager');
mock.module('../../../../components/TabManager', () => ({
	...actualTabManager,
	useTabManager: () => ({
		activeTabId: 'tab',
		getExplorerState: () => explorerState,
		updateExplorerState: () => {}
	})
}));

const {act, useState} = await import('react');
const {createRoot} = await import('react-dom/client');
const {useTable} = await import('./useTable');

(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

const file = (name: string) =>
	({
		id: name,
		name,
		extension: null,
		kind: 'File',
		size: 1,
		created_at: '2026-01-01T00:00:00Z',
		modified_at: '2026-01-01T00:00:00Z',
		tags: []
	}) as never;
const files = [file('b'), file('a')];

const seen: unknown[] = [];
let rerender: () => void = () => {};
function List() {
	const [, setTick] = useState(0);
	rerender = () => setTick((tick) => tick + 1);
	const {table} = useTable(files, 800);
	seen.push(table.getRowModel().rows);
	return null;
}

describe('useTable', () => {
	// ListView publishes its rows to the Explorer on every change. New rows on
	// every render made that a render loop that kept a CPU core busy.
	it('keeps the same rows when re-rendered with the same input', async () => {
		explorerState.listSorting = [{id: 'modified', desc: true}];
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<List />));
		await act(async () => rerender());
		await act(async () => rerender());

		expect(seen.length).toBeGreaterThanOrEqual(3);
		expect(seen.at(-1)).toBe(seen[0]);
		await act(async () => root.unmount());
	});

	it('keeps the same rows without a saved sort', async () => {
		explorerState.listSorting = undefined;
		seen.length = 0;
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<List />));
		await act(async () => rerender());

		expect(seen.at(-1)).toBe(seen[0]);
		await act(async () => root.unmount());
	});
});

describe('sameFiles', () => {
	it('compares the files, not the array', () => {
		const {sameFiles} = actualContext;
		expect(sameFiles([...files], [...files])).toBe(true);
		expect(sameFiles([], [])).toBe(true);
		expect(sameFiles(files, [files[1], files[0]])).toBe(false);
		expect(sameFiles(files, [files[0], file('a')])).toBe(false);
		expect(sameFiles(files, files.slice(1))).toBe(false);
	});
});
