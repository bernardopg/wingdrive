import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface DraggedItem {
	type: 'file' | 'space-item' | 'space-group';
	data: any;
}

interface SidebarStore {
	// Persisted state
	currentSpaceId: string | null;
	setCurrentSpace: (id: string | null) => void;

	folderTreePositions: Record<string, string | null>;
	setFolderTreePosition: (spaceId: string, afterGroupId: string | null | undefined) => void;
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

export const useSidebarStore = create<SidebarStore>()(
	persist(
		(set) => ({
			// Persisted
			currentSpaceId: null,
			setCurrentSpace: (id) => set({ currentSpaceId: id }),

			dynamicItemOrder: {},
			folderTreePositions: {},
			setFolderTreePosition: (spaceId, afterGroupId) => set((state) => {
				const positions = {...state.folderTreePositions};
				if (afterGroupId === undefined) delete positions[spaceId];
				else positions[spaceId] = afterGroupId;
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
			partialize: (state) => ({
				currentSpaceId: state.currentSpaceId,
				dynamicItemOrder: state.dynamicItemOrder,
				folderTreePositions: state.folderTreePositions,
			}),
		}
	)
);
