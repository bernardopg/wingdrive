import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {describe, expect, it} from 'bun:test';

if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

const {useSyncPreferencesStore} = await import('../stores/syncPreferences');

describe('syncPreferences store', () => {
	it('defaults to auto switching on sync', () => {
		expect(useSyncPreferencesStore.getState().autoSwitchOnSync).toBe(true);
	});

	it('updates the auto switch preference', () => {
		const store = useSyncPreferencesStore.getState();
		store.setAutoSwitchOnSync(false);
		expect(useSyncPreferencesStore.getState().autoSwitchOnSync).toBe(false);

		store.setAutoSwitchOnSync(true);
		expect(useSyncPreferencesStore.getState().autoSwitchOnSync).toBe(true);
	});
});
