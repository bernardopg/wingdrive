import { GlobalRegistrator } from "@happy-dom/global-registrator";
import { beforeEach, describe, expect, it } from "bun:test";

if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

const { useViewPreferencesStore } = await import("../stores/viewPreferences");

describe("useViewPreferencesStore", () => {
	beforeEach(() => {
		localStorage.removeItem("wingdrive-view-preferences");
		useViewPreferencesStore.setState({ preferences: {} });
	});

	it("returns undefined for unknown wing items", () => {
		expect(useViewPreferencesStore.getState().getPreferences("missing")).toBeUndefined();
	});

	it("stores preferences merged with defaults", () => {
		useViewPreferencesStore.getState().setPreferences("wing-1", {
			viewMode: "list",
			viewSettings: { gridSize: 200 },
		});

		expect(useViewPreferencesStore.getState().getPreferences("wing-1")).toEqual({
			viewMode: "list",
			viewSettings: {
				gridSize: 200,
				gapSize: 16,
				showFileSize: true,
				calculateFolderSizes: false,
				showHiddenFiles: false,
				columnWidth: 256,
			},
		});
	});

	it("preserves existing view settings when updating a subset", () => {
		const store = useViewPreferencesStore.getState();
		store.setPreferences("wing-1", {
			viewMode: "grid",
			viewSettings: { showHiddenFiles: true, gapSize: 24 },
		});
		store.setPreferences("wing-1", {
			viewSettings: { gridSize: 96 },
		});

		expect(useViewPreferencesStore.getState().getPreferences("wing-1")).toEqual({
			viewMode: "grid",
			viewSettings: {
				gridSize: 96,
				gapSize: 24,
				showFileSize: true,
				calculateFolderSizes: false,
				showHiddenFiles: true,
				columnWidth: 256,
			},
		});
	});

	it("keeps preferences isolated per wing item", () => {
		const store = useViewPreferencesStore.getState();
		store.setPreferences("a", { viewMode: "media" });
		store.setPreferences("b", { viewMode: "column" });

		expect(useViewPreferencesStore.getState().getPreferences("a")?.viewMode).toBe("media");
		expect(useViewPreferencesStore.getState().getPreferences("b")?.viewMode).toBe("column");
	});
});
