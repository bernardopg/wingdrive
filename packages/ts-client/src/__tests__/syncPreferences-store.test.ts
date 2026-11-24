import { GlobalRegistrator } from "@happy-dom/global-registrator";
import { beforeEach, describe, expect, it } from "bun:test";

if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

const { useSyncPreferencesStore } = await import("../stores/syncPreferences");

describe("useSyncPreferencesStore", () => {
	beforeEach(() => {
		localStorage.removeItem("wingdrive-sync-preferences");
		useSyncPreferencesStore.setState({ autoSwitchOnSync: true });
	});

	it("defaults to automatically switching on sync", () => {
		expect(useSyncPreferencesStore.getState().autoSwitchOnSync).toBe(true);
	});

	it("updates the auto-switch preference", () => {
		useSyncPreferencesStore.getState().setAutoSwitchOnSync(false);
		expect(useSyncPreferencesStore.getState().autoSwitchOnSync).toBe(false);

		useSyncPreferencesStore.getState().setAutoSwitchOnSync(true);
		expect(useSyncPreferencesStore.getState().autoSwitchOnSync).toBe(true);
	});
});
