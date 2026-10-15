import {describe, expect, it, mock} from 'bun:test';
import {QueryClient, QueryClientProvider} from '@tanstack/react-query';
import type {ReactElement} from 'react';
import {renderToStaticMarkup} from 'react-dom/server';

const mutation = {mutateAsync: async () => ({}), isPending: false};
const appConfig = {
	preferences: {theme: 'dark', language: 'en'},
	job_logging: {enabled: true, include_debug: false},
	log_level: 'info',
	telemetry_enabled: true
};
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
	useLibraryQuery: () => ({data: libraryConfig, refetch: () => {}, isLoading: false}),
	useLibraryMutation: () => mutation,
	useWingDriveClient: () => ({getCurrentLibraryId: () => 'library'})
}));

mock.module('../../contexts/PlatformContext', () => ({
	usePlatform: () => ({platform: 'web', openLink: () => {}})
}));

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

	it('pages render in Portuguese when the language is pt', async () => {
		await i18n.changeLanguage('pt');
		try {
			expect(render(<AdvancedSettings />)).toContain('Incluir logs de depuração');
		} finally {
			await i18n.changeLanguage('en');
		}
	});
});
