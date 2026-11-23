import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {describe, expect, it} from 'bun:test';

if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

const {useViewPreferencesStore} = await import('../stores/viewPreferences');

describe('viewPreferences store', () => {
	it('returns undefined for unknown wing items', () => {
		expect(useViewPreferencesStore.getState().getPreferences('missing')).toBeUndefined();
	});

	it('stores defaults and merges partial updates', () => {
		const store = useViewPreferencesStore.getState();
		store.setPreferences('wing-a', {viewMode: 'list'});
		expect(store.getPreferences('wing-a')).toEqual({
			viewMode: 'list',
			viewSettings: {
				gridSize: 120,
				gapSize: 16,
				showFileSize: true,
				calculateFolderSizes: false,
				showHiddenFiles: false,
				columnWidth: 256,
			},
		});

		store.setPreferences('wing-a', {viewSettings: {gridSize: 200, showHiddenFiles: true}});
		expect(store.getPreferences('wing-a')).toEqual({
			viewMode: 'list',
			viewSettings: {
				gridSize: 200,
				gapSize: 16,
				showFileSize: true,
				calculateFolderSizes: false,
				showHiddenFiles: true,
				columnWidth: 256,
			},
		});
	});

	it('keeps preferences isolated per wing item', () => {
		const store = useViewPreferencesStore.getState();
		store.setPreferences('wing-b', {viewMode: 'media'});
		store.setPreferences('wing-c', {viewMode: 'column', viewSettings: {columnWidth: 320}});

		expect(store.getPreferences('wing-b')?.viewMode).toBe('media');
		expect(store.getPreferences('wing-c')?.viewMode).toBe('column');
		expect(store.getPreferences('wing-c')?.viewSettings.columnWidth).toBe(320);
		expect(store.getPreferences('wing-b')?.viewSettings.columnWidth).toBe(256);
	});
});
