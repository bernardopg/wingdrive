import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {describe, expect, it, mock} from 'bun:test';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

const {act} = await import('react');
const {createRoot} = await import('react-dom/client');
const {WingDriveProvider, useNormalizedQuery} = await import('@wingdrive/ts-client/hooks');

(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

const unsubscribe = mock(() => {});
const subscribeFiltered = mock(async () => unsubscribe);
const client = {
	getCurrentLibraryId: () => 'library',
	on: () => {},
	off: () => {},
	execute: async () => ({files: []}),
	subscribeFiltered
} as never;

function Listing({enabled, live}: {enabled: boolean; live?: boolean}) {
	useNormalizedQuery({
		query: 'files.directory_listing',
		input: {path: {Physical: {device_slug: 'local', path: '/tmp'}}} as never,
		resourceType: 'file',
		pathScope: {Physical: {device_slug: 'local', path: '/tmp'}},
		enabled,
		live
	});
	return null;
}

describe('useNormalizedQuery subscriptions', () => {
	it('opens no subscription while the query is disabled, and one once enabled', async () => {
		const container = document.createElement('div');
		const root = createRoot(container);
		const render = (enabled: boolean) =>
			root.render(
				<WingDriveProvider client={client}>
					<Listing enabled={enabled} />
				</WingDriveProvider>
			);

		await act(async () => render(false));
		expect(subscribeFiltered).not.toHaveBeenCalled();

		await act(async () => render(true));
		expect(subscribeFiltered).toHaveBeenCalledTimes(1);

		await act(async () => render(false));
		expect(unsubscribe).toHaveBeenCalledTimes(1);

		await act(async () => root.unmount());
	});

	it('opens no subscription for an enabled query that is not live', async () => {
		subscribeFiltered.mockClear();
		const container = document.createElement('div');
		const root = createRoot(container);

		await act(async () =>
			root.render(
				<WingDriveProvider client={client}>
					<Listing enabled live={false} />
				</WingDriveProvider>
			)
		);
		expect(subscribeFiltered).not.toHaveBeenCalled();

		await act(async () => root.unmount());
	});
});
