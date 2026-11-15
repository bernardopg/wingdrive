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
});
