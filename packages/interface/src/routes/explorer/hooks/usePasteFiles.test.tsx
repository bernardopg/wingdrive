import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {beforeEach, describe, expect, it, mock} from 'bun:test';

if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

const toasts: string[] = [];
let clipboardContents = {operation: 'copy' as 'copy' | 'cut', files: [] as unknown[], sourcePath: null};
let finishPasteError: unknown = null;
let readFilesError: unknown = null;
const finished: unknown[] = [];
const opened: unknown[] = [];

const actualI18n = await import('../../../i18n');
mock.module('../../../i18n', () => ({
	...actualI18n,
	useTranslation: () => ({t: (key: string, opts?: unknown) => (opts ? `${key} ${JSON.stringify(opts)}` : key)})
}));
const actualClipboard = await import('../../../hooks/useClipboard');
mock.module('../../../hooks/useClipboard', () => ({
	...actualClipboard,
	useClipboard: () => ({
		readFiles: async () => {
			if (readFilesError) throw readFilesError;
			return clipboardContents;
		},
		finishPaste: async (contents: unknown) => {
			finished.push(contents);
			if (finishPasteError) throw finishPasteError;
		}
	})
}));
mock.module('../../../components/modals/FileOperationModal', () => ({
	useFileOperationDialog: () => (opts: unknown) => opened.push(opts)
}));
const actualPrimitives = await import('@wingdrive/primitives');
mock.module('@wingdrive/primitives', () => ({
	...actualPrimitives,
	toast: {...actualPrimitives.toast, error: (msg: string) => void toasts.push(msg)}
}));

const {act} = await import('react');
const {createRoot} = await import('react-dom/client');
const {usePasteFiles} = await import('./usePasteFiles');

(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

const local = (path: string) => ({Physical: {device_slug: 'local', path}}) as never;
let paste: ReturnType<typeof usePasteFiles>;
function Probe() {
	paste = usePasteFiles();
	return null;
}

beforeEach(() => {
	toasts.length = 0;
	opened.length = 0;
	finished.length = 0;
	finishPasteError = null;
	readFilesError = null;
	clipboardContents = {operation: 'copy', files: [local('/a')], sourcePath: null};
});

describe('usePasteFiles', () => {
	it('ignores paste when no destination or clipboard is empty', async () => {
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<Probe />));
		await act(async () => paste(null));
		expect(opened).toEqual([]);

		clipboardContents = {operation: 'copy', files: [], sourcePath: null};
		await act(async () => paste(local('/dst')));
		expect(opened).toEqual([]);
		await act(async () => root.unmount());
	});

	it('opens a copy operation and finishes paste after completion', async () => {
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<Probe />));
		await act(async () => paste(local('/dst')));
		expect(opened).toHaveLength(1);
		const opts = opened[0] as any;
		expect(opts.operation).toBe('copy');
		expect(opts.sources).toEqual([local('/a')]);
		expect(opts.destination).toEqual(local('/dst'));
		expect(opts.confirmOnlyOnConflict).toBe(true);

		await act(async () => opts.onComplete());
		expect(finished).toEqual([clipboardContents]);
		expect(toasts).toEqual([]);
		await act(async () => root.unmount());
	});

	it('maps cut clipboard to move and reports finish-paste errors', async () => {
		finishPasteError = new Error('boom');
		clipboardContents = {operation: 'cut', files: [local('/a')], sourcePath: local('/src')};
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<Probe />));
		await act(async () => paste(local('/dst')));
		const opts = opened[0] as any;
		expect(opts.operation).toBe('move');
		await act(async () => opts.onComplete());
		await act(async () => new Promise((r) => setTimeout(r, 0)));
		expect(toasts[0]).toContain('errors.couldNotUpdateClipboard');
		await act(async () => root.unmount());
	});

	it('reports clipboard read failures', async () => {
		readFilesError = new Error('denied');
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<Probe />));
		await act(async () => paste(local('/dst')));
		expect(opened).toEqual([]);
		expect(toasts[0]).toContain('errors.couldNotPasteFiles');
		await act(async () => root.unmount());
	});
});
