import {GlobalRegistrator} from '@happy-dom/global-registrator';
import {describe, expect, it} from 'bun:test';

// Registered once per test process and never unregistered: unregistering closes
// the window whose timers React's scheduler keeps using in the next test file.
if (!GlobalRegistrator.isRegistered) GlobalRegistrator.register();

// The sidebar as WingDrive 3.3.1 saved it, before the store is created.
localStorage.setItem(
	'wingdrive-sidebar',
	JSON.stringify({
		state: {
			currentSpaceId: 'wing-1',
			dynamicItemOrder: {devices: ['b', 'a']},
			folderTreePositions: {'wing-1': 'group-2'}
		},
		version: 0
	})
);

const {migrateSidebar, useSidebarStore} = await import('../stores/sidebar');

describe('sidebar store', () => {
	it('keeps the wing selected before the rename', () => {
		const state = useSidebarStore.getState();
		expect(state.currentWingId).toBe('wing-1');
		expect(state.dynamicItemOrder).toEqual({devices: ['b', 'a']});
		expect(state.folderTreePositions).toEqual({'wing-1': 'group-2'});
		expect('currentSpaceId' in state).toBe(false);

		state.setCurrentWing('wing-2');
		const saved = JSON.parse(localStorage.getItem('wingdrive-sidebar')!);
		expect(saved.version).toBe(1);
		expect(saved.state.currentWingId).toBe('wing-2');
		expect(saved.state.currentSpaceId).toBeUndefined();
	});

	it('migrates saved state of any shape', () => {
		expect(migrateSidebar(null, 0)).toEqual({} as never);
		expect(migrateSidebar({currentSpaceId: null}, 0)).toEqual({currentWingId: null} as never);
		expect(migrateSidebar({currentWingId: 'w'}, 1)).toEqual({currentWingId: 'w'} as never);
	});

	it('updates folder tree positions and dynamic order', () => {
		const state = useSidebarStore.getState();
		state.setFolderTreePosition('wing-1', 'group-1');
		expect(useSidebarStore.getState().folderTreePositions).toEqual({'wing-1': 'group-1'});
		state.setFolderTreePosition('wing-1', null);
		expect(useSidebarStore.getState().folderTreePositions).toEqual({'wing-1': null});
		state.setFolderTreePosition('wing-1', undefined);
		expect(useSidebarStore.getState().folderTreePositions).toEqual({});

		state.setDynamicItemOrder('g1', ['a', 'b']);
		expect(useSidebarStore.getState().dynamicItemOrder.g1).toEqual(['a', 'b']);
	});

	it('toggles and collapses groups', () => {
		const state = useSidebarStore.getState();
		state.expandAll();
		expect(useSidebarStore.getState().collapsedGroups.size).toBe(0);
		state.toggleGroup('g1');
		expect(useSidebarStore.getState().collapsedGroups.has('g1')).toBe(true);
		state.toggleGroup('g1');
		expect(useSidebarStore.getState().collapsedGroups.has('g1')).toBe(false);
		state.collapseAll(['g1', 'g2']);
		expect([...useSidebarStore.getState().collapsedGroups].sort()).toEqual(['g1', 'g2']);
		state.expandAll();
		expect(useSidebarStore.getState().collapsedGroups.size).toBe(0);
	});

	it('tracks dragged item ephemerally', () => {
		const item = {type: 'file' as const, data: {id: 'f1'}};
		useSidebarStore.getState().setDraggedItem(item);
		expect(useSidebarStore.getState().draggedItem).toEqual(item);
		useSidebarStore.getState().setDraggedItem(null);
		expect(useSidebarStore.getState().draggedItem).toBeNull();
	});
});
