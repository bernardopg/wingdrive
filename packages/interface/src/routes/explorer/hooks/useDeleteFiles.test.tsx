import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {beforeEach, describe, expect, it, mock} from 'bun:test';

if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

const confirmations: any[] = [];
const refetches: number[] = [];
const journaled: Array<[string, string]> = [];
const adminReports: any[] = [];
const reported: any[] = [];
let mutationPending = false;
let mutateResult: any = {job_id: 'job-1'};
let waitResult: any = {
	receipt: {id: 'receipt-1'},
	result: {status: 'completed', output: {type: 'FileDelete', data: {deleted_count: 1, failed_count: 0, total_bytes: 10}}}
};
let waitError: unknown = null;

const actualI18n = await import('../../../i18n');
mock.module('../../../i18n', () => ({
	...actualI18n,
	useTranslation: () => ({t: (key: string, opts?: unknown) => (opts ? `${key} ${JSON.stringify(opts)}` : key)})
}));
const actualWingDrive = await import('../../../contexts/WingDriveContext');
mock.module('../../../contexts/WingDriveContext', () => ({
	...actualWingDrive,
	useLibraryMutation: () => ({
		get isPending() {
			return mutationPending;
		},
		mutateAsync: async (input: unknown) => {
			if (typeof mutateResult === 'function') return mutateResult(input);
			return mutateResult;
		}
	})
}));
mock.module('../../../components/modals/DeleteConfirmationModal', () => ({
	useDeleteConfirmationDialog: () => (opts: unknown) => confirmations.push(opts)
}));
const actualWait = await import('../../../hooks/useWaitForJob');
mock.module('../../../hooks/useWaitForJob', () => ({
	...actualWait,
	useWaitForJob: () => async (start: () => Promise<unknown>) => {
		const out = await start();
		if (waitError) throw waitError;
		return typeof waitResult === 'function' ? waitResult(out) : waitResult;
	}
}));
const actualRefetch = await import('../../../hooks/useRefetchFileListings');
mock.module('../../../hooks/useRefetchFileListings', () => ({
	...actualRefetch,
	useRefetchFileListings: () => () => refetches.push(1)
}));
const actualUndo = await import('../../../hooks/useUndo');
mock.module('../../../hooks/useUndo', () => ({
	...actualUndo,
	useUndo: () => ({
		isLocalPath: (p: any) => 'Physical' in p,
		recordJournaled: (label: string, id: string) => journaled.push([label, id])
	})
}));
const actualAdmin = await import('../../../hooks/adminRetry');
mock.module('../../../hooks/adminRetry', () => ({
	...actualAdmin,
	useAdminRetry: () => ({
		canRetry: (detail: string) => detail.includes('Permission'),
		report: (...args: unknown[]) => adminReports.push(args)
	})
}));
const actualReport = await import('../../../hooks/reportFileOperation');
mock.module('../../../hooks/reportFileOperation', () => ({
	...actualReport,
	reportFileOperation: (outcome: unknown) => reported.push(outcome)
}));

const {act} = await import('react');
const {createRoot} = await import('react-dom/client');
const {useDeleteFiles} = await import('./useDeleteFiles');

(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

function file(id: string, local = true): any {
	return {
		id,
		name: id,
		is_local: local,
		kind: 'File',
		wing_path: local
			? {Physical: {device_slug: 'local', path: `/home/me/${id}`}}
			: {Cloud: {service: 'S3', identifier: 'b', path: id}}
	};
}

let api: ReturnType<typeof useDeleteFiles>;
function Probe() {
	api = useDeleteFiles();
	return null;
}

beforeEach(() => {
	confirmations.length = 0;
	refetches.length = 0;
	journaled.length = 0;
	adminReports.length = 0;
	reported.length = 0;
	mutationPending = false;
	waitError = null;
	mutateResult = {job_id: 'job-1'};
	waitResult = {
		receipt: {id: 'receipt-1'},
		result: {status: 'completed', output: {type: 'FileDelete', data: {deleted_count: 1, failed_count: 0, total_bytes: 10}}}
	};
});

describe('useDeleteFiles', () => {
	it('returns false for empty, pending, or virtual targets', async () => {
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<Probe />));
		expect(await api.deleteFiles([], false)).toBe(false);
		mutationPending = true;
		expect(await api.deleteFiles([file('a')], false)).toBe(false);
		mutationPending = false;
		const virtual = {...file('a'), _virtual: {type: 'device'}};
		expect(await api.deleteFiles([virtual], false)).toBe(false);
		expect(confirmations).toEqual([]);
		await act(async () => root.unmount());
	});

	it('cancels without mutating and confirms to delete, refetch, report and journal', async () => {
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<Probe />));

		const cancelled = api.deleteFiles([file('a')], false);
		expect(confirmations).toHaveLength(1);
		await act(async () => confirmations[0].onCancelled());
		expect(await cancelled).toBe(false);
		expect(refetches).toEqual([]);

		const confirmed = api.deleteFiles([file('a')], false);
		expect(confirmations).toHaveLength(2);
		expect(confirmations[1].permanent).toBe(false);
		await act(async () => confirmations[1].onConfirm());
		expect(await confirmed).toBe(true);
		expect(refetches).toEqual([1]);
		expect(reported[0].status).toBe('success');
		expect(journaled).toEqual([['delete', 'receipt-1']]);
		await act(async () => root.unmount());
	});

	it('does not journal permanent deletes or cloud-only deletes', async () => {
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<Probe />));

		let p = act(async () => api.deleteFiles([file('a')], true));
		await act(async () => confirmations[0].onConfirm());
		await p;
		expect(journaled).toEqual([]);

		p = act(async () => api.deleteFiles([file('c', false)], false));
		await act(async () => confirmations[1].onConfirm());
		await p;
		expect(journaled).toEqual([]);
		await act(async () => root.unmount());
	});

	it('reports partial delete failures through the normal outcome path', async () => {
		waitResult = {
			receipt: {id: 'receipt-1'},
			result: {status: 'completed', output: {type: 'FileDelete', data: {deleted_count: 0, failed_count: 1, total_bytes: 0}}}
		};
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<Probe />));
		const p = api.deleteFiles([file('a')], false);
		await act(async () => confirmations[0].onConfirm());
		expect(await p).toBe(false);
		expect(reported).toHaveLength(1);
		expect(reported[0].status).toBe('failed');
		expect(journaled).toEqual([]);
		await act(async () => root.unmount());
	});

	it('reports thrown errors and offers admin retry for local paths', async () => {
		waitError = new Error('disk');
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<Probe />));
		const p = api.deleteFiles([file('a')], false);
		await act(async () => confirmations[0].onConfirm());
		expect(await p).toBe(false);
		expect(adminReports).toHaveLength(1);
		expect(adminReports[0][0]).toContain('errors.failedToDelete');
		expect(adminReports[0][1]).toEqual({type: 'delete', paths: ['/home/me/a']});
		await act(async () => root.unmount());
	});
});
