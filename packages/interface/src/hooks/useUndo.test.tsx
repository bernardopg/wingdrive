import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {beforeEach, describe, expect, it, mock} from 'bun:test';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

let libraryId: string | null = 'lib-1';
const undoCalls: string[] = [];
let undoOutcome: {status: string; output?: unknown; error?: string} = {
	status: 'completed',
	output: {type: 'FileUndo', data: {skipped: []}}
};
let refetches = 0;
const toasts: Array<[string, string]> = [];

// Bun module mocks leak into other test files, so keep the real exports.
const actualWingDrive = await import('../contexts/WingDriveContext');
mock.module('../contexts/WingDriveContext', () => ({
	...actualWingDrive,
	useWingDriveClient: () => ({getCurrentLibraryId: () => libraryId}),
	useLibraryMutation: () => ({
		mutateAsync: async ({job_id}: {job_id: string}) => {
			undoCalls.push(job_id);
			return {id: `undo-${job_id}`};
		}
	}),
	useLibraryQuery: () => ({data: [{slug: 'my-laptop', is_current: true}]})
}));
const actualRefetch = await import('./useRefetchFileListings');
mock.module('./useRefetchFileListings', () => ({
	...actualRefetch,
	useRefetchFileListings: () => () => {
		refetches += 1;
	}
}));
const actualWait = await import('./useWaitForJob');
mock.module('./useWaitForJob', () => ({
	...actualWait,
	useWaitForJob:
		() =>
		async (dispatch: () => Promise<{id: string}>) => ({
			receipt: await dispatch(),
			result: undoOutcome
		})
}));
const actualPrimitives = await import('@wingdrive/primitives');
mock.module('@wingdrive/primitives', () => ({
	...actualPrimitives,
	toast: {
		...actualPrimitives.toast,
		success: (message: string) => void toasts.push(['success', message]),
		error: (message: string) => void toasts.push(['error', message])
	}
}));

const {act} = await import('react');
const {createRoot} = await import('react-dom/client');
const {useUndo} = await import('./useUndo');
const {useUndoStore} = await import('./undoStore');

(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

let undo: ReturnType<typeof useUndo>;
function Probe() {
	undo = useUndo();
	return null;
}
async function mount() {
	const root = createRoot(document.createElement('div'));
	await act(async () => root.render(<Probe />));
	return root;
}

beforeEach(() => {
	useUndoStore.setState({entries: [], busy: false});
	libraryId = 'lib-1';
	undoCalls.length = 0;
	refetches = 0;
	toasts.length = 0;
	undoOutcome = {status: 'completed', output: {type: 'FileUndo', data: {skipped: []}}};
});

describe('useUndo', () => {
	it('tells local paths apart by slug', async () => {
		const root = await mount();
		const at = (device_slug: string) => ({Physical: {device_slug, path: '/a'}}) as never;
		expect(undo.isLocalPath(at('local'))).toBe(true);
		expect(undo.isLocalPath(at('my-laptop'))).toBe(true);
		expect(undo.isLocalPath(at('phone'))).toBe(false);
		expect(undo.isLocalPath({Content: {content_id: 'c'}} as never)).toBe(false);
		await act(async () => root.unmount());
	});

	it('undoes journaled jobs newest first and reports the step', async () => {
		const root = await mount();
		expect(undo.canUndo).toBe(false);
		await act(async () => undo.recordJournaled('rename', 'job-a'));
		await act(async () => undo.recordJournaled('copy', ['job-b', null, 'job-c', undefined]));
		await act(async () => undo.recordJournaled('nothing', [null]));
		expect(useUndoStore.getState().entries.map((entry) => entry.label)).toEqual(['rename', 'copy']);
		expect(undo.canUndo).toBe(true);

		await act(async () => undo.undo());
		expect(undoCalls).toEqual(['job-c', 'job-b']);
		expect(refetches).toBe(1);
		expect(toasts.at(-1)?.[0]).toBe('success');

		await act(async () => undo.undo());
		expect(undoCalls).toEqual(['job-c', 'job-b', 'job-a']);
		await act(async () => undo.undo());
		expect(toasts).toHaveLength(2);
		await act(async () => root.unmount());
	});

	it('refetches and reports a failed undo', async () => {
		const root = await mount();
		undoOutcome = {status: 'failed', error: 'moved since'};
		await act(async () => undo.recordJournaled('trash', 'job-t'));
		await act(async () => undo.undo());
		expect(refetches).toBe(1);
		expect(toasts.at(-1)?.[0]).toBe('error');
		await act(async () => root.unmount());
	});

	it('records nothing and undoes nothing without a library', async () => {
		libraryId = null;
		const root = await mount();
		await act(async () => undo.record('x', async () => {}));
		expect(useUndoStore.getState().entries).toEqual([]);
		await act(async () => undo.undo());
		expect(toasts).toEqual([]);
		await act(async () => root.unmount());
	});
});
