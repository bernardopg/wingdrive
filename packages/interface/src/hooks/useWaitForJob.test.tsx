import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {describe, expect, it, mock} from 'bun:test';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

let emit: ((event: unknown) => void) | undefined;
let subscribeFails = false;
let unsubscribed = 0;
const client = {
	subscribeFiltered: async (_filter: unknown, handler: (event: unknown) => void) => {
		if (subscribeFails) throw new Error('offline');
		emit = handler;
		return () => {
			unsubscribed += 1;
		};
	}
};

// Bun module mocks leak into other test files, so keep the real exports.
const actualWingDrive = await import('../contexts/WingDriveContext');
mock.module('../contexts/WingDriveContext', () => ({
	...actualWingDrive,
	useWingDriveClient: () => client
}));

const {act} = await import('react');
const {createRoot} = await import('react-dom/client');
const {useWaitForJob} = await import('./useWaitForJob');

(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

let waitForJob: ReturnType<typeof useWaitForJob>;
function Probe() {
	waitForJob = useWaitForJob();
	return null;
}
const root = createRoot(document.createElement('div'));
await act(async () => root.render(<Probe />));

describe('useWaitForJob', () => {
	it('resolves with the terminal event of the dispatched job', async () => {
		const pending = waitForJob(async () => ({id: 'job-1'}));
		await new Promise((resolve) => setTimeout(resolve, 0));
		emit!({JobProgress: {job_id: 'job-1'}});
		emit!({JobCompleted: {job_id: 'other', output: {type: 'Empty'}}});
		emit!(null);
		emit!({JobCompleted: {job_id: 'job-1', output: {type: 'Empty'}}});
		expect(await pending).toEqual({
			receipt: {id: 'job-1'},
			result: {status: 'completed', output: {type: 'Empty'}} as never
		});
		expect(unsubscribed).toBe(1);
	});

	it('keeps events that arrive before the receipt', async () => {
		const result = await waitForJob(async () => {
			emit!({JobFailed: {job_id: 'fast', error: 'denied'}});
			return {id: 'fast'};
		});
		expect(result.result).toEqual({status: 'failed', error: 'denied'});

		const cancelled = await waitForJob(async () => {
			emit!({JobCancelled: {job_id: 'gone'}});
			return {id: 'gone'};
		});
		expect(cancelled.result).toEqual({status: 'cancelled'});
	});

	it('times out a job that never reports back', async () => {
		const result = await waitForJob(async () => ({id: 'silent'}), 20);
		expect(result.result).toEqual({status: 'timeout'});
	});

	it('treats a failed subscription as a timeout and passes dispatch errors on', async () => {
		subscribeFails = true;
		const result = await waitForJob(async () => ({id: 'x'}));
		expect(result.result).toEqual({status: 'timeout'});
		subscribeFails = false;

		const before = unsubscribed;
		await expect(
			waitForJob(async () => {
				throw new Error('rejected');
			})
		).rejects.toThrow('rejected');
		expect(unsubscribed).toBe(before + 1);
	});
});
