import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {describe, expect, it, mock} from 'bun:test';
import {QueryClient, QueryClientProvider} from '@tanstack/react-query';
import {act, type ReactElement} from 'react';
import {renderToStaticMarkup} from 'react-dom/server';

if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register({url: 'http://localhost/'});
(globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;

const mutation = {mutateAsync: async () => ({}), isPending: false};
const appConfig = {
	preferences: {theme: 'dark', language: 'en'},
	job_logging: {enabled: true, include_debug: false},
	log_level: 'info',
	telemetry_enabled: true
};
let configAvailable = true;
const libraryConfig = {
	generate_thumbnails: true,
	thumbnail_quality: 85,
	enable_ai_tagging: false,
	sync_enabled: true,
	encryption_enabled: false,
	auto_track_system_volumes: true,
	auto_track_external_volumes: false
};

// Bun module mocks leak into other test files, so keep the real exports and override only these
const actualContext = await import('../../contexts/WingDriveContext');
mock.module('../../contexts/WingDriveContext', () => ({
	...actualContext,
	useCoreQuery: () => ({data: appConfig, refetch: () => {}}),
	useCoreMutation: () => mutation,
	useLibraryQuery: () => ({data: configAvailable ? libraryConfig : undefined, refetch: () => {}, isLoading: false}),
	useLibraryMutation: () => mutation,
	useWingDriveClient: () => ({getCurrentLibraryId: () => 'library'})
}));

mock.module('../../contexts/PlatformContext', () => ({
	usePlatform: () => ({platform: 'web', openLink: () => {}})
}));

mock.module('@wingdrive/assets/images', () => ({WingDriveLogo: 'logo.svg'}));
const {createRoot} = await import('react-dom/client');
const {Settings} = await import('../../routes/settings');
const {LibrarySettings} = await import('./LibrarySettings');
const {AppearanceSettings} = await import('./AppearanceSettings');
const {PrivacySettings} = await import('./PrivacySettings');
const {AdvancedSettings} = await import('./AdvancedSettings');
const {i18n} = await import('../../i18n');

function render(element: ReactElement) {
	return renderToStaticMarkup(
		<QueryClientProvider client={new QueryClient()}>{element}</QueryClientProvider>
	);
}

describe('settings pages only offer settings with an effect', () => {
	it('library settings drop thumbnail, AI, sync and encryption switches', () => {
		const html = render(<LibrarySettings />);
		for (const text of ['Encryption', 'Sync Enabled', 'AI Tagging', 'Thumbnail Quality', 'Generate Thumbnails']) {
			expect(html).not.toContain(text);
		}
		expect(html).toContain('System Volumes');
		expect(html).toContain('External Volumes');
	});

	it('offers a retry instead of editable defaults when library settings fail to load', () => {
		configAvailable = false;
		try {
			const html = render(<LibrarySettings />);
			expect(html).toContain('Could not load settings.');
			expect(html).toContain('Try again');
			expect(html).not.toContain('role="switch"');
		} finally {
			configAvailable = true;
		}
	});

	it('appearance offers the language picker', () => {
		const html = render(<AppearanceSettings />);
		expect(html).toContain('<select');
		expect(html).toContain('Português (Brasil)');
	});

	it('privacy states that nothing is collected', () => {
		const html = render(<PrivacySettings />);
		expect(html).not.toContain('type="checkbox"');
		expect(html).toContain('does not collect usage data');
	});

	it('advanced keeps only the debug log switch', () => {
		const html = render(<AdvancedSettings />);
		expect(html).not.toContain('Daemon Log Level');
		expect(html).not.toContain('Enable Job Logging');
		expect(html).toContain('Include Debug Logs');
	});

	it('keeps the native-sized selector, sidebar, URL and current page in sync', async () => {
		const originalUrl = window.location.href;
		window.history.replaceState(null, '', '/settings/library');
		const host = document.createElement('div');
		const root = createRoot(host);
		try {
			await act(async () => root.render(
				<QueryClientProvider client={new QueryClient()}><Settings /></QueryClientProvider>
			));
			const nav = host.querySelector('nav')!;
			const main = host.querySelector('main')!;
			const select = nav.querySelector('select')!;
			expect(select.options).toHaveLength(9);
			expect(select.value).toBe('library');

			main.scrollTop = 80;
			await act(async () => {
				select.value = 'privacy';
				select.dispatchEvent(new window.Event('change', {bubbles: true}));
			});
			expect(main.scrollTop).toBe(0);
			expect(window.location.pathname).toBe('/settings/privacy');
			expect(nav.querySelector('[aria-current="page"]')?.textContent).toBe('Privacy');
			expect(main.textContent).toContain('does not collect usage data');

			await act(async () => {
				window.history.replaceState(null, '', '/settings/advanced');
				window.dispatchEvent(new window.PopStateEvent('popstate'));
			});
			expect(select.value).toBe('advanced');
			expect(main.textContent).toContain('Include Debug Logs');
		} finally {
			await act(async () => root.unmount());
			window.history.replaceState(null, '', originalUrl);
		}
	});

	it('pages render in Portuguese when the language is pt', async () => {
		await i18n.changeLanguage('pt');
		try {
			expect(render(<AdvancedSettings />)).toContain('Incluir logs de depuração');
		} finally {
			await i18n.changeLanguage('en');
		}
	});
});
