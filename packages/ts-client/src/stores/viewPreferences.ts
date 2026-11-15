import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface ViewSettings {
	gridSize: number;
	gapSize: number;
	showFileSize: boolean;
	calculateFolderSizes: boolean;
	showHiddenFiles: boolean;
	columnWidth: number;
}

interface WingItemViewPreferences {
	viewMode: 'grid' | 'list' | 'media' | 'column' | 'size' | 'knowledge';
	viewSettings: ViewSettings;
}

interface ViewPreferencesStore {
	preferences: Record<string, WingItemViewPreferences>;
	getPreferences: (wingItemId: string) => WingItemViewPreferences | undefined;
	setPreferences: (wingItemId: string, prefs: Partial<WingItemViewPreferences>) => void;
}

const DEFAULT_VIEW_SETTINGS: ViewSettings = {
	gridSize: 120,
	gapSize: 16,
	showFileSize: true,
	calculateFolderSizes: false,
	showHiddenFiles: false,
	columnWidth: 256,
};

const DEFAULT_PREFERENCES: WingItemViewPreferences = {
	viewMode: 'grid',
	viewSettings: DEFAULT_VIEW_SETTINGS,
};

export const useViewPreferencesStore = create<ViewPreferencesStore>()(
	persist(
		(set, get) => ({
			preferences: {},
			getPreferences: (wingItemId) => {
				return get().preferences[wingItemId];
			},
			setPreferences: (wingItemId, prefs) =>
				set((state) => ({
					preferences: {
						...state.preferences,
						[wingItemId]: {
							...DEFAULT_PREFERENCES,
							...state.preferences[wingItemId],
							...prefs,
							viewSettings: {
								...DEFAULT_VIEW_SETTINGS,
								...state.preferences[wingItemId]?.viewSettings,
								...(prefs.viewSettings || {}),
							},
						},
					},
				})),
		}),
		{
			name: 'wingdrive-view-preferences',
		}
	)
);
