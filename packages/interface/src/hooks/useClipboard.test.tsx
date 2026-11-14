import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {beforeEach, describe, expect, it, mock} from 'bun:test';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

interface FakePlatform {
	writeFileClipboard?: (paths: string[], cut: boolean) => Promise<void>;
	readFileClipboard?: () => Promise<[string[], boolean]>;
}
let platform: FakePlatform = {};
const writes: Array<[string[], boolean]> = [];
let systemClipboard: [string[], boolean] = [[], false];
let writeFails = false;
const toasts: string[] = [];

// Bun module mocks leak into other test files, so keep the real exports.
const actualPlatform = await import('../contexts/PlatformContext');
mock.module('../contexts/PlatformContext', () => ({
	...actualPlatform,
	usePlatform: () => platform
}));
const actualPrimitives = await import('@wingdrive/primitives');
mock.module('@wingdrive/primitives', () => ({
	...actualPrimitives,
	toast: {...actualPrimitives.toast, error: (message: string) => void toasts.push(message)}
}));

const {act} = await import('react');
const {createRoot} = await import('react-dom/client');
const {useClipboard, useClipboardStore, useIsCut} = await import('./useClipboard');

(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

const local = (path: string) => ({Physical: {device_slug: 'local', path}}) as never;
const cloud = {Cloud: {service: 'S3', identifier: 'b', path: 'x'}} as never;
const fullPlatform: FakePlatform = {
	writeFileClipboard: async (paths, cut) => {
		if (writeFails) throw new Error('no clipboard');
		writes.push([paths, cut]);
		systemClipboard = [paths, cut];
	},
	readFileClipboard: async () => systemClipboard
};

let clipboard: ReturnType<typeof useClipboard>;
let cutA = false;
function Probe() {
	clipboard = useClipboard();
	cutA = useIsCut(local('/a'));
	return null;
}
async function mount() {
	const root = createRoot(document.createElement('div'));
	await act(async () => root.render(<Probe />));
	return root;
}
const settle = () => act(async () => new Promise((resolve) => setTimeout(resolve, 0)));

beforeEach(() => {
	useClipboardStore.getState().clearClipboard();
	platform = {};
	writes.length = 0;
	toasts.length = 0;
	systemClipboard = [[], false];
	writeFails = false;
});

describe('useClipboard', () => {
	it('keeps copies in the app when the platform has no file clipboard', async () => {
		const root = await mount();
		expect(clipboard.hasClipboard()).toBe(false);
		expect(clipboard.canPaste()).toBe(false);
		await act(async () => clipboard.copyFiles([local('/a')], local('/')));
		expect(clipboard.operation).toBe('copy');
		expect(clipboard.files).toEqual([local('/a')]);
		expect(clipboard.sourcePath).toEqual(local('/'));
		expect(clipboard.canPaste()).toBe(true);
		expect(cutA).toBe(false);
		expect(await clipboard.readFiles()).toMatchObject({operation: 'copy'});
		await act(async () => clipboard.finishPaste(await clipboard.readFiles()));
		// A copy stays for the next paste.
		expect(clipboard.hasClipboard()).toBe(true);
		await act(async () => root.unmount());
	});

	it('exports local files to the system clipboard and dims cut items', async () => {
		platform = fullPlatform;
		const root = await mount();
		await act(async () => clipboard.cutFiles([local('/a'), local('/b')]));
		await settle();
		expect(writes).toEqual([[['/a', '/b'], true]]);
		expect(useClipboardStore.getState().exportedToSystem).toBe(true);
		expect(cutA).toBe(true);

		// Once exported, the system clipboard is the source of truth.
		systemClipboard = [['/c'], false];
		const contents = await clipboard.readFiles();
		expect(contents).toEqual({operation: 'copy', files: [local('/c')], sourcePath: null});
		await act(async () => root.unmount());
	});

	it('does not export remote files or files copied without the local flag', async () => {
		platform = fullPlatform;
		const root = await mount();
		await act(async () => clipboard.copyFiles([cloud]));
		await act(async () => clipboard.copyFiles([local('/a')], null, false));
		await settle();
		expect(writes).toEqual([]);
		// Not exported yet: the app's own clipboard answers.
		expect((await clipboard.readFiles()).files).toEqual([local('/a')]);
		await act(async () => root.unmount());
	});

	it('reads files another app put on the system clipboard', async () => {
		platform = fullPlatform;
		const root = await mount();
		expect(clipboard.canPaste()).toBe(true);
		expect((await clipboard.readFiles()).files).toEqual([]);
		systemClipboard = [['/x', '/y'], true];
		expect(await clipboard.readFiles()).toEqual({
			operation: 'cut',
			files: [local('/x'), local('/y')],
			sourcePath: null
		});
		await act(async () => root.unmount());
	});

	it('pasting a cut clears both clipboards when they still hold it', async () => {
		platform = fullPlatform;
		const root = await mount();
		await act(async () => clipboard.cutFiles([local('/a')]));
		await settle();
		await act(async () => clipboard.finishPaste(await clipboard.readFiles()));
		expect(writes.at(-1)).toEqual([[], false]);
		expect(clipboard.hasClipboard()).toBe(false);

		// Something else was cut meanwhile: leave it.
		await act(async () => clipboard.cutFiles([local('/b')]));
		await settle();
		systemClipboard = [['/other'], true];
		await act(async () =>
			clipboard.finishPaste({operation: 'cut', files: [local('/a')], sourcePath: null})
		);
		expect(clipboard.files).toEqual([local('/b')]);
		await act(async () => root.unmount());
	});

	it('reports a failed export', async () => {
		platform = fullPlatform;
		writeFails = true;
		const root = await mount();
		await act(async () => clipboard.copyFiles([local('/a')]));
		await settle();
		expect(toasts).toHaveLength(1);
		expect(useClipboardStore.getState().exportedToSystem).toBe(false);
		await act(async () => root.unmount());
	});
});
