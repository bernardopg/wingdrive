import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface DraggedItem {
	type: 'file' | 'wing-item' | 'wing-group';
	data: any;
}

interface SidebarStore {
	// Persisted state
	currentWingId: string | null;
	setCurrentWing: (id: string | null) => void;

	folderTreePositions: Record<string, string | null>;
	setFolderTreePosition: (wingId: string, afterGroupId: string | null | undefined) => void;
	dynamicItemOrder: Record<string, string[]>;
	setDynamicItemOrder: (groupId: string, ids: string[]) => void;

	// Ephemeral state
	collapsedGroups: Set<string>;
	toggleGroup: (groupId: string) => void;
	collapseAll: (groupIds: string[]) => void;
	expandAll: () => void;

	// Drag state
	draggedItem: DraggedItem | null;
	setDraggedItem: (item: DraggedItem | null) => void;
}

type PersistedSidebar = Pick<
	SidebarStore,
	'currentWingId' | 'dynamicItemOrder' | 'folderTreePositions'
>;

/**
 * Upgrades the saved sidebar. Version 0 (WingDrive 3.3.1 and older) saved the
 * selected wing as `currentSpaceId`.
 */
export function migrateSidebar(saved: unknown, version: number): PersistedSidebar {
	const state = {...(saved as Record<string, unknown> | null)};
	if (version < 1 && 'currentSpaceId' in state) {
		state.currentWingId = state.currentSpaceId ?? null;
		delete state.currentSpaceId;
	}
	return state as unknown as PersistedSidebar;
}

export const useSidebarStore = create<SidebarStore>()(
	persist(
		(set) => ({
			// Persisted
			currentWingId: null,
			setCurrentWing: (id) => set({ currentWingId: id }),

			dynamicItemOrder: {},
			folderTreePositions: {},
			setFolderTreePosition: (wingId, afterGroupId) => set((state) => {
				const positions = {...state.folderTreePositions};
				if (afterGroupId === undefined) delete positions[wingId];
				else positions[wingId] = afterGroupId;
				return {folderTreePositions: positions};
			}),
			setDynamicItemOrder: (groupId, ids) => set((state) => ({
				dynamicItemOrder: {...state.dynamicItemOrder, [groupId]: ids},
			})),

			// Ephemeral
			collapsedGroups: new Set(),
			toggleGroup: (groupId) =>
				set((state) => {
					const newSet = new Set(state.collapsedGroups);
					if (newSet.has(groupId)) {
						newSet.delete(groupId);
					} else {
						newSet.add(groupId);
					}
					return { collapsedGroups: newSet };
				}),
			collapseAll: (groupIds) =>
				set({
					collapsedGroups: new Set(groupIds),
				}),
			expandAll: () => set({ collapsedGroups: new Set() }),

			// Drag
			draggedItem: null,
			setDraggedItem: (item) => set({ draggedItem: item }),
		}),
		{
			name: 'wingdrive-sidebar',
			version: 1,
			migrate: migrateSidebar,
			partialize: (state) => ({
				currentWingId: state.currentWingId,
				dynamicItemOrder: state.dynamicItemOrder,
				folderTreePositions: state.folderTreePositions,
			}),
		}
	)
);
