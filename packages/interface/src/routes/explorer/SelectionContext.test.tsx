import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {afterEach, beforeEach, describe, expect, it, mock} from 'bun:test';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

type Listener = () => void;

/** What the provider asked of the hooks around it. */
const calls = {
	platformIds: [] as string[][],
	menus: [] as Array<Array<{id: string; enabled: boolean}>>,
	renames: [] as Array<{target: unknown; new_name: string}>,
	journaled: [] as Array<[string, string]>,
	refetches: 0,
	adminReports: [] as Array<{message: string; retry: unknown}>
};
let tabSelections: Record<string, string[]> = {};
let activeTabId = 'tab-1';
let renameResult: {status: string; output?: unknown; error?: string} = {
	status: 'completed',
	output: {type: 'FileMove', data: {failed_count: 0}}
};
const clientListeners = new Map<string, Set<Listener>>();
const client = {
	on(event: string, listener: Listener) {
		if (!clientListeners.has(event)) clientListeners.set(event, new Set());
		clientListeners.get(event)!.add(listener);
	},
	off(event: string, listener: Listener) {
		clientListeners.get(event)?.delete(listener);
	}
};

// Bun module mocks leak into other test files, so keep the real exports.
const actualI18n = await import('../../i18n');
mock.module('../../i18n', () => ({
	...actualI18n,
	useTranslation: () => ({
		t: (key: string, options?: Record<string, unknown>) =>
			options ? `${key} ${JSON.stringify(options)}` : key
	})
}));
const actualTabs = await import('../../components/TabManager');
mock.module('../../components/TabManager', () => ({
	...actualTabs,
	useTabManager: () => ({
		activeTabId,
		getSelectionIds: (tabId: string) => tabSelections[tabId] ?? [],
		updateSelectionIds: (tabId: string, ids: string[]) => {
			tabSelections = {...tabSelections, [tabId]: ids};
		}
	})
}));
const actualPlatform = await import('../../contexts/PlatformContext');
mock.module('../../contexts/PlatformContext', () => ({
	...actualPlatform,
	usePlatform: () => platform
}));
const platform = {
	setSelectedFileIds: async (ids: string[]) => {
		calls.platformIds.push(ids);
	},
	updateMenuItems: (items: Array<{id: string; enabled: boolean}>) => {
		calls.menus.push(items);
	}
};
const renameMutation = {
	mutateAsync: async (input: {target: unknown; new_name: string}) => {
		calls.renames.push(input);
		return {job_id: 'job-1'};
	}
};
const actualWingDrive = await import('../../contexts/WingDriveContext');
mock.module('../../contexts/WingDriveContext', () => ({
	...actualWingDrive,
	useLibraryMutation: () => renameMutation,
	useWingDriveClient: () => client
}));
const actualClipboard = await import('../../hooks/useClipboard');
const clipboard = {};
mock.module('../../hooks/useClipboard', () => ({
	...actualClipboard,
	useClipboard: () => clipboard
}));
const actualAdmin = await import('../../hooks/adminRetry');
const adminRetry = {
	report: (message: string, retry: unknown) => {
		calls.adminReports.push({message, retry});
	}
};
mock.module('../../hooks/adminRetry', () => ({
	...actualAdmin,
	useAdminRetry: () => adminRetry
}));
const actualRefetch = await import('../../hooks/useRefetchFileListings');
const refetch = () => {
	calls.refetches += 1;
};
mock.module('../../hooks/useRefetchFileListings', () => ({
	...actualRefetch,
	useRefetchFileListings: () => refetch
}));
const actualUndo = await import('../../hooks/useUndo');
const undo = {
	recordJournaled: (label: string, id: string) => {
		calls.journaled.push([label, id]);
	}
};
mock.module('../../hooks/useUndo', () => ({...actualUndo, useUndo: () => undo}));
const actualWait = await import('../../hooks/useWaitForJob');
const waitForJob = async (start: () => Promise<unknown>) => {
	await start();
	return {receipt: {id: 'receipt-1'}, result: renameResult};
};
mock.module('../../hooks/useWaitForJob', () => ({
	...actualWait,
	useWaitForJob: () => waitForJob
}));

const {act} = await import('react');
const {createRoot} = await import('react-dom/client');
const {SelectionProvider, useSelection} = await import('./SelectionContext');
const {requestReveal} = await import('./pendingReveal');

(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

function file(id: string, name = id, extension: string | null = 'txt') {
	return {
		id,
		name,
		extension,
		kind: 'File',
		is_local: true,
		wing_path: {Physical: {device_slug: 'local', path: `/home/me/${name}.${extension}`}}
	} as never;
}
const a = file('a');
const b = file('b');
const c = file('c');
const d = file('d');
const files = [a, b, c, d];

let selection: ReturnType<typeof useSelection>;
function Probe() {
	selection = useSelection();
	return null;
}

let root: ReturnType<typeof createRoot> | undefined;
async function mount(isActiveTab = true) {
	root = createRoot(document.createElement('div'));
	await act(async () => {
		root.render(
			<SelectionProvider isActiveTab={isActiveTab}>
				<Probe />
			</SelectionProvider>
		);
	});
}
const ids = () => selection.selectedFiles.map((f) => f.id);
const run = (fn: () => void) => act(async () => fn());

beforeEach(() => {
	tabSelections = {};
	activeTabId = 'tab-1';
	calls.platformIds = [];
	calls.menus = [];
	calls.renames = [];
	calls.journaled = [];
	calls.refetches = 0;
	calls.adminReports = [];
	renameResult = {status: 'completed', output: {type: 'FileMove', data: {failed_count: 0}}};
});

afterEach(async () => {
	const mounted = root;
	root = undefined;
	if (mounted) await act(async () => mounted.unmount());
});

describe('SelectionProvider', () => {
	it('selects one file, toggles with multi and extends ranges', async () => {
		await mount();
		await run(() => selection.selectFile(b, files));
		expect(ids()).toEqual(['b']);
		expect(selection.focusedIndex).toBe(1);
		expect(tabSelections['tab-1']).toEqual(['b']);
		expect(selection.isSelected('b')).toBe(true);

		await run(() => selection.selectFile(d, files, true));
		expect(ids()).toEqual(['b', 'd']);
		await run(() => selection.selectFile(b, files, true));
		expect(ids()).toEqual(['d']);

		// The anchor is the last clicked item (b, even though it was just
		// unselected), and a range replaces a single selection.
		await run(() => selection.selectFile(a, files, false, true));
		expect(ids()).toEqual(['a', 'b']);
		expect(selection.focusedIndex).toBe(0);

		// With several selected, a range adds what is missing (Finder behavior).
		await run(() => selection.selectFile(a, files));
		await run(() => selection.selectFile(c, files, true));
		await run(() => selection.selectFile(d, files, false, true));
		expect(ids()).toEqual(['a', 'c', 'd']);
	});

	it('selects all, clears and moves the focus inside the list', async () => {
		await mount();
		await run(() => selection.selectAll(files));
		expect(ids()).toEqual(['a', 'b', 'c', 'd']);
		await run(() => selection.clearSelection());
		expect(ids()).toEqual([]);
		expect(selection.focusedIndex).toBe(-1);

		await run(() => selection.moveFocus('down', []));
		expect(selection.focusedIndex).toBe(-1);
		await run(() => selection.moveFocus('down', files));
		expect(ids()).toEqual(['a']);
		await run(() => selection.moveFocus('right', files));
		expect(ids()).toEqual(['b']);
		await run(() => selection.moveFocus('up', files));
		await run(() => selection.moveFocus('left', files));
		expect(ids()).toEqual(['a']);
		// At the edge nothing changes.
		await run(() => selection.moveFocus('up', files));
		expect(selection.focusedIndex).toBe(0);
		await run(() => selection.setFocusedIndex(3));
		await run(() => selection.moveFocus('down', files));
		expect(selection.focusedIndex).toBe(3);
	});

	it('tells the platform about the selection only for the active tab', async () => {
		await mount();
		await run(() => selection.setSelectedFiles([a]));
		expect(calls.platformIds.at(-1)).toEqual(['a']);
		const menu = calls.menus.at(-1)!;
		expect(menu.find((item) => item.id === 'rename')?.enabled).toBe(true);
		await act(async () => root!.unmount());

		calls.platformIds = [];
		calls.menus = [];
		await mount(false);
		await run(() => selection.setSelectedFiles([a, b]));
		expect(calls.platformIds).toEqual([]);
		expect(calls.menus).toEqual([]);
	});

	it('clears the selection when the library changes', async () => {
		await mount();
		await run(() => selection.selectFile(a, files));
		await run(() => {
			for (const listener of clientListeners.get('library-changed') ?? []) listener();
		});
		expect(ids()).toEqual([]);
		expect(selection.focusedIndex).toBe(-1);
	});

	it('renames the sole selection and records the journaled undo', async () => {
		await mount();
		await run(() => selection.selectFile(a, files));
		await run(() => selection.startRename('a'));
		expect(selection.isRenaming).toBe(true);
		await act(async () => selection.saveRename('renamed.txt'));
		expect(calls.renames).toEqual([
			{target: (a as {wing_path: unknown}).wing_path, new_name: 'renamed.txt'}
		]);
		expect(calls.journaled).toEqual([['undo.rename', 'receipt-1']]);
		expect(calls.refetches).toBe(1);
		expect(selection.renamingFileId).toBeNull();
	});

	it('skips empty or unchanged names and cancels on request', async () => {
		await mount();
		await run(() => selection.selectFile(a, files));
		await run(() => selection.startRename('a'));
		await act(async () => selection.saveRename('a.txt'));
		expect(calls.renames).toEqual([]);
		expect(selection.renamingFileId).toBeNull();

		await run(() => selection.startRename('a'));
		await act(async () => selection.saveRename('   '));
		expect(calls.renames).toEqual([]);

		await run(() => selection.startRename('a'));
		await run(() => selection.cancelRename());
		expect(selection.isRenaming).toBe(false);
		// Nothing to save when no rename is running.
		await act(async () => selection.saveRename('x.txt'));
		expect(calls.renames).toEqual([]);
	});

	it('renaming an unselected file selects it first; several selected cannot rename', async () => {
		await mount();
		await run(() => selection.selectAll([a, b]));
		await run(() => selection.startRename('a'));
		expect(selection.isRenaming).toBe(false);

		await run(() => selection.startRename('c', c));
		expect(ids()).toEqual(['c']);
		expect(selection.renamingFileId).toBe('c');

		// Changing the selection away from the renamed file ends the rename.
		await run(() => selection.selectFile(d, files));
		expect(selection.renamingFileId).toBeNull();
	});

	it('keeps editing and reports a failed rename with an admin retry', async () => {
		await mount();
		renameResult = {status: 'failed', error: 'permission denied'};
		await run(() => selection.selectFile(a, files));
		await run(() => selection.startRename('a'));
		let thrown: unknown;
		await act(async () => {
			try {
				await selection.saveRename('b2.txt');
			} catch (error) {
				thrown = error;
			}
		});
		expect((thrown as Error).message).toBe('permission denied');
		expect(selection.renamingFileId).toBe('a');
		expect(calls.refetches).toBe(1);
		expect(calls.adminReports).toHaveLength(1);
		expect(calls.adminReports[0]!.retry).toEqual({
			type: 'rename',
			path: '/home/me/a.txt',
			new_name: 'b2.txt'
		});

		// A move that left files behind counts as incomplete.
		renameResult = {status: 'completed', output: {type: 'FileMove', data: {failed_count: 1}}};
		await act(async () => {
			try {
				await selection.saveRename('b3.txt');
			} catch (error) {
				thrown = error;
			}
		});
		expect((thrown as Error).message).toBe('errors.renameIncomplete');
	});

	it('restores the stored selection from a listing and prunes missing entries', async () => {
		tabSelections = {'tab-1': ['b', 'x']};
		await mount();
		expect(selection.isSelected('b')).toBe(true);
		// An empty listing is still loading: the stored ids stay.
		await run(() => selection.restoreSelectionFromFiles([]));
		expect(tabSelections['tab-1']).toEqual(['b', 'x']);

		await run(() => selection.restoreSelectionFromFiles(files));
		expect(ids()).toEqual(['b']);
		expect(tabSelections['tab-1']).toEqual(['b']);
	});

	it('selects revealed files and starts the rename they asked for', async () => {
		await mount();
		requestReveal('/home/me/c.txt', {rename: true});
		await run(() => selection.restoreSelectionFromFiles(files));
		expect(ids()).toEqual(['c']);
		expect(selection.focusedIndex).toBe(2);
		expect(selection.renamingFileId).toBe('c');
		expect(tabSelections['tab-1']).toEqual(['c']);
	});

	it('pendingRename holds a name until the listing shows it', async () => {
		await mount();
		expect(selection.pendingRename).toBeNull();
		await run(() => selection.renameWhenListed('New Folder'));
		expect(selection.pendingRename).toBe('New Folder');
	});

	it('useSelection outside the provider throws', async () => {
		function Outside() {
			useSelection();
			return null;
		}
		root = createRoot(document.createElement('div'));
		const errors = console.error;
		console.error = () => {};
		try {
			let thrown: unknown;
			try {
				await act(async () => {
					root!.render(<Outside />);
				});
			} catch (error) {
				thrown = error;
			}
			expect((thrown as Error).message).toBe('useSelection must be used within SelectionProvider');
		} finally {
			console.error = errors;
		}
	});
});
