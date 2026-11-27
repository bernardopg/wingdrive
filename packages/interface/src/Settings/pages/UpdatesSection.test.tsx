import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {describe, expect, it, mock} from 'bun:test';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();
(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

let platform: Record<string, unknown> = {};

// Bun module mocks leak into other test files, so keep the real exports.
const actualPlatform = await import('../../contexts/PlatformContext');
mock.module('../../contexts/PlatformContext', () => ({
	...actualPlatform,
	usePlatform: () => platform
}));

const {act} = await import('react');
const {createRoot} = await import('react-dom/client');
const {UpdatesSection} = await import('./UpdatesSection');
const {i18n} = await import('../../i18n');

const settings = {
	keep_in_background: true,
	start_at_login: true,
	start_minimized: true,
	check_for_updates: true,
	terminal_command: null
};

async function render() {
	await i18n.changeLanguage('en');
	const host = document.createElement('div');
	document.body.appendChild(host);
	const root = createRoot(host);
	await act(async () => {
		root.render(<UpdatesSection desktopSettings={settings} onSettingChange={() => {}} />);
	});
	return {host, root};
}

function button(host: HTMLElement, text: string) {
	return [...host.querySelectorAll('button')].find(b => b.textContent === text);
}

describe('updates section', () => {
	it('explains where updates come from on unsupported systems', async () => {
		platform = {getUpdaterInfo: async () => ({channel: 'unsupported', current_version: '3.4.0'})};
		const {host, root} = await render();
		expect(host.textContent).toContain('Updates on this system come from new downloads on wingdrive.org');
		act(() => root.unmount());
	});
	it('tells package-manager installs about new releases without installing them', async () => {
		platform = {
			getUpdaterInfo: async () => ({channel: 'package_manager', current_version: '3.4.0'}),
			checkForUpdate: async () => ({version: '3.5.0', date: null, notes: null}),
			installUpdate: async () => {
				throw new Error('must not be called');
			}
		};
		const {host, root} = await render();
		expect(host.textContent).toContain('WingDrive 3.4.0');
		expect(host.textContent).toContain('the package manager you used to install it');
		expect(host.textContent).toContain('Check for updates automatically');
		await act(async () => button(host, 'Check for updates')!.click());
		expect(host.textContent).toContain('WingDrive 3.5.0 is available. Update it with your package manager');
		expect(button(host, 'Install and restart')).toBeUndefined();
		act(() => root.unmount());
	});

	it('checks, offers the new version and installs it on self-updating copies', async () => {
		const installed: string[] = [];
		platform = {
			getUpdaterInfo: async () => ({channel: 'self_update', current_version: '3.3.1'}),
			checkForUpdate: async () => ({version: '3.4.0', date: null, notes: null}),
			installUpdate: async (onProgress: (p: {downloaded: number; total: number}) => void) => {
				onProgress({downloaded: 5, total: 10});
				installed.push('3.4.0');
			}
		};
		const {host, root} = await render();
		expect(host.textContent).toContain('Check for updates automatically');
		await act(async () => button(host, 'Check for updates')!.click());
		expect(host.textContent).toContain('WingDrive 3.4.0 is available');
		await act(async () => button(host, 'Install and restart')!.click());
		expect(installed).toEqual(['3.4.0']);
		act(() => root.unmount());
	});

	it('shows a retry after failing to load update information', async () => {
		let requests = 0;
		platform = {
			getUpdaterInfo: async () => {
				if (++requests === 1) throw new Error('Unavailable');
				return {channel: 'package_manager', current_version: '3.4.0'};
			}
		};
		const {host, root} = await render();
		expect(host.querySelector('[role="alert"]')?.textContent).toBe('Could not load settings.');
		await act(async () => button(host, 'Try again')!.click());
		expect(host.textContent).toContain('WingDrive 3.4.0');
		act(() => root.unmount());
	});

	it('shows nothing where the platform has no updater', async () => {
		platform = {};
		const {host, root} = await render();
		expect(host.textContent).toBe('');
		act(() => root.unmount());
	});
});
