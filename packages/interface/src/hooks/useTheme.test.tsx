import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {afterEach, beforeEach, describe, expect, it, mock} from 'bun:test';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

let config: {preferences?: {theme?: string; language?: string}} | undefined;
let refetches = 0;
let handler: ((event: unknown) => void) | undefined;
let unsubscribed = 0;
const client = {
	subscribeFiltered: async (_filter: unknown, onEvent: (event: unknown) => void) => {
		handler = onEvent;
		return () => {
			unsubscribed += 1;
		};
	}
};

// Bun module mocks leak into other test files, so keep the real exports.
const actualWingDrive = await import('../contexts/WingDriveContext');
mock.module('../contexts/WingDriveContext', () => ({
	...actualWingDrive,
	useWingDriveClient: () => client,
	useCoreQuery: () => ({
		data: config,
		refetch: () => {
			refetches += 1;
		}
	})
}));

// Silence the hook's subscription logging.
console.log = () => {};

const {act} = await import('react');
const {createRoot} = await import('react-dom/client');
const {applyTheme, themePreviewClass, useIsDarkTheme, useTheme, useThemedIcon} = await import(
	'./useTheme'
);

(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

let systemDark = true;
const mediaListeners = new Set<(event: {matches: boolean}) => void>();
window.matchMedia = ((query: string) => ({
	matches: systemDark,
	media: query,
	addEventListener: (_: string, listener: (event: {matches: boolean}) => void) =>
		mediaListeners.add(listener),
	removeEventListener: (_: string, listener: (event: {matches: boolean}) => void) =>
		mediaListeners.delete(listener)
})) as never;

const classes = () => [...document.documentElement.classList];

function Probe() {
	useTheme();
	return null;
}

beforeEach(() => {
	config = undefined;
	refetches = 0;
	handler = undefined;
	unsubscribed = 0;
	systemDark = true;
	document.documentElement.className = '';
});

afterEach(() => {
	mediaListeners.clear();
});

describe('applyTheme', () => {
	it('swaps the theme class and leaves dark without one', () => {
		applyTheme('nord');
		expect(classes()).toEqual(['nord-theme']);
		applyTheme('light');
		expect(classes()).toEqual(['vanilla-theme']);
		applyTheme('dark');
		expect(classes()).toEqual([]);
		systemDark = false;
		applyTheme('system');
		expect(classes()).toEqual(['vanilla-theme']);
	});

	it('names a preview class for every theme', () => {
		expect(themePreviewClass('dracula')).toBe('dracula-theme');
		expect(themePreviewClass('dark')).toBe('dark-theme');
		systemDark = false;
		expect(themePreviewClass('system')).toBe('vanilla-theme');
	});
});

describe('useTheme', () => {
	it('applies the configured theme and language and refetches on config events', async () => {
		config = {preferences: {theme: 'mocha', language: 'pt'}};
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<Probe />));
		expect(classes()).toEqual(['mocha-theme']);
		expect(document.documentElement.lang).toBe('pt-BR');

		await act(async () => {
			handler!({ConfigChanged: {field: 'theme'}});
			handler!({ConfigChanged: {field: 'language'}});
			handler!({ConfigChanged: {field: 'other'}});
			handler!({JobStarted: {}});
		});
		expect(refetches).toBe(2);
		// A fixed theme does not follow the system.
		expect(mediaListeners.size).toBe(0);

		await act(async () => root.unmount());
		expect(unsubscribed).toBe(1);
		await act(async () => {
			const {applyLanguage} = await import('../i18n');
			applyLanguage('en');
		});
	});

	it('follows the system scheme while the theme is "system"', async () => {
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<Probe />));
		expect(classes()).toEqual([]);
		expect(mediaListeners.size).toBe(1);
		await act(async () => {
			for (const listener of mediaListeners) listener({matches: false});
		});
		expect(classes()).toEqual(['vanilla-theme']);
		await act(async () => root.unmount());
		expect(mediaListeners.size).toBe(0);
	});
});

describe('useIsDarkTheme', () => {
	it('measures the app background and picks light icons on a light one', async () => {
		let dark: boolean | undefined;
		let themed: ((url: string) => string) | undefined;
		function Icons() {
			dark = useIsDarkTheme();
			themed = useThemedIcon();
			return null;
		}
		const root = createRoot(document.createElement('div'));
		await act(async () => root.render(<Icons />));
		expect(typeof dark).toBe('boolean');
		const url = '/icons/Folder.png';
		expect(themed!(url)).toBe(dark ? url : themed!(url));

		// A class change re-measures and notifies.
		await act(async () => {
			document.documentElement.classList.add('vanilla-theme');
			await new Promise((resolve) => setTimeout(resolve, 0));
		});
		await act(async () => {
			for (const listener of mediaListeners) listener({matches: true});
		});
		await act(async () => root.unmount());
	});
});
