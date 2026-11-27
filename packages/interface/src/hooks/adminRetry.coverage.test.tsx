import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {describe, expect, it, mock} from 'bun:test';
import type {AdminOperation, AdminRunOutcome} from '../contexts/PlatformContext';

if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();
(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

let outcome: AdminRunOutcome = {status: 'completed'};
let failure = false;
const refresh = mock(() => {});
const errors = mock((..._args: unknown[]) => {});
const success = mock((..._args: unknown[]) => {});
const info = mock((..._args: unknown[]) => {});
const realPrimitives = await import('@wingdrive/primitives');
mock.module('@wingdrive/primitives', () => ({...realPrimitives, toast: {...realPrimitives.toast, error: errors, success, info}}));
const realPlatform = await import('../contexts/PlatformContext');
mock.module('../contexts/PlatformContext', () => ({...realPlatform, usePlatform: () => ({runAsAdmin: async (_operation: AdminOperation) => {
	if (failure) throw new Error('failed');
	return outcome;
}})}));
mock.module('./useRefetchFileListings', () => ({useRefetchFileListings: () => refresh}));
const {useAdminRetry} = await import('./adminRetry');
const {act} = await import('react');
const {createRoot} = await import('react-dom/client');
async function renderRetry() {
	let value: ReturnType<typeof useAdminRetry> | undefined;
	function Probe() { value = useAdminRetry(); return null; }
	const root = createRoot(document.createElement('div'));
	await act(async () => root.render(<Probe />));
	if (!value) throw new Error('hook did not render');
	return {current: value, cleanup: () => act(() => root.unmount())};
}

const operation: AdminOperation = {type: 'delete', paths: ['/synthetic/file']};

describe('administrator retry', () => {
	it('reports cancellation without refreshing', async () => {
		refresh.mockClear(); info.mockClear();
		outcome = {status: 'cancelled'};
		const result = await renderRetry();
		expect(await result.current.runAsAdmin(operation)).toBe(false);
		expect(refresh).not.toHaveBeenCalled();
		expect(info).toHaveBeenCalledTimes(1);
		result.cleanup();
	});

	it('refreshes and reports partial failures with their paths', async () => {
		refresh.mockClear(); errors.mockClear();
		outcome = {status: 'partial', failures: [{path: '/synthetic/file', error: 'denied'}]};
		const result = await renderRetry();
		expect(await result.current.runAsAdmin(operation)).toBe(true);
		expect(refresh).toHaveBeenCalledTimes(1);
		expect(String(errors.mock.calls[0][0])).toContain('/synthetic/file: denied');
		result.cleanup();
	});

	it('reports a rejected administrator operation', async () => {
		refresh.mockClear(); errors.mockClear(); failure = true;
		const result = await renderRetry();
		try {
			expect(await result.current.runAsAdmin(operation)).toBe(false);
			expect(refresh).not.toHaveBeenCalled();
			expect(String(errors.mock.calls[0][0])).toContain('failed');
		} finally { failure = false; result.cleanup(); }
	});
});
