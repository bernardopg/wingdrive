import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {expect, mock, test} from 'bun:test';

if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();
(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

const listMountableVolumes = mock(async () => [{name: 'Fixture phone', uri: 'mtp://fixture/'}]);

const platformContext = await import('../../contexts/PlatformContext');
mock.module('../../contexts/PlatformContext', () => ({
	...platformContext,
	usePlatform: () => ({listNetworkMounts: async () => [], listMountableVolumes})
}));
mock.module('../TabManager/useSidebarNavigate', () => ({
	useSidebarNavigate: () => () => {}
}));

const {QueryClient, QueryClientProvider} = await import('@tanstack/react-query');
const {act} = await import('react');
const {createRoot} = await import('react-dom/client');
const {i18n} = await import('../../i18n');
const {NetworkSection} = await import('./NetworkSection');

test('reports a failed device list and replaces the retry with devices after recovery', async () => {
	await i18n.changeLanguage('en');
	listMountableVolumes.mockImplementationOnce(async () => {
		throw new Error('volume monitor unavailable');
	});

	const client = new QueryClient({defaultOptions: {queries: {retry: false}}});
	const host = document.createElement('div');
	document.body.appendChild(host);
	const root = createRoot(host);
	const label = 'Could not list devices. Retry';
	const retryButton = () =>
		[...host.querySelectorAll('button')].find(button => button.textContent === label);

	async function waitForText(text: string) {
		for (let attempt = 0; attempt < 20 && !host.textContent?.includes(text); attempt++) {
			await act(async () => { await new Promise(resolve => setTimeout(resolve, 10)); });
		}
		expect(host.textContent).toContain(text);
	}

	try {
		await act(async () => {
			root.render(
				<QueryClientProvider client={client}>
					<NetworkSection groupId="network" />
				</QueryClientProvider>
			);
		});
		await waitForText(label);
		expect(retryButton()?.title).toContain('volume monitor unavailable');
		await act(async () => retryButton()!.click());
		await waitForText('Fixture phone');
		expect(retryButton()).toBeUndefined();
		expect(listMountableVolumes).toHaveBeenCalledTimes(2);
	} finally {
		act(() => root.unmount());
		client.clear();
		host.remove();
	}
});
