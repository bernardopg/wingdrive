import {describe, expect, test} from 'bun:test';
import {orderDynamicItems, rememberDynamicOrder, reorderItems, reorderLayoutItems, sidebarSections, isSidebarSortable} from './reorderItems';

describe('sidebar item order', () => {
	const items = [{id: 'a'}, {id: 'b'}, {id: 'c'}];
	test('moves an item without changing the original order', () => {
		expect(reorderItems(items, 'a', 'c')?.map((item) => item.id)).toEqual(['b', 'c', 'a']);
		expect(items.map((item) => item.id)).toEqual(['a', 'b', 'c']);
	});
	test('ignores drops outside the same list and unchanged positions', () => {
		expect(reorderItems(items, 'a', 'other-group')).toBeNull();
		expect(reorderItems(items, 'a', 'a')).toBeNull();
	});
});

describe('dynamic sidebar items', () => {
	test('restores saved order and appends newly discovered items', () => {
		const items = [{id: 'new'}, {id: 'a'}, {id: 'b'}];
		expect(orderDynamicItems(items, ['b', 'missing', 'a']).map((item) => item.id)).toEqual(['b', 'a', 'new']);
	});
	test('keeps absent items remembered when the visible items are reordered', () => {
		const saved = rememberDynamicOrder(['b', 'a'], ['a', 'offline', 'b']);
		expect(saved).toEqual(['b', 'offline', 'a']);
		expect(orderDynamicItems([{id: 'offline'}, {id: 'a'}, {id: 'b'}], saved).map((item) => item.id)).toEqual(saved);
	});
});


describe('space layout ordering', () => {
	const layout = {
		space_items: [{id: 'root-a'}, {id: 'root-b'}],
		groups: [
			{group: {id: 'group-a'}, items: [{id: 'a'}, {id: 'b'}]},
			{group: {id: 'group-b'}, items: [{id: 'other'}]}
		]
	};
	test('reorders a group without changing the root or other groups', () => {
		const result = reorderLayoutItems(layout, 'a', 'b');
		expect(result?.groupId).toBe('group-a');
		expect(result?.itemIds).toEqual(['b', 'a']);
		expect(result?.layout.space_items).toEqual(layout.space_items);
		expect(result?.layout.groups[1]).toEqual(layout.groups[1]);
		expect(layout.groups[0].items.map((item) => item.id)).toEqual(['a', 'b']);
	});
	test('reorders root items and rejects cross-group drops', () => {
		expect(reorderLayoutItems(layout, 'root-a', 'root-b')?.groupId).toBeNull();
		expect(reorderLayoutItems(layout, 'a', 'other')).toBeNull();
		expect(reorderLayoutItems(layout, 'root-a', 'a')).toBeNull();
	});
});


test('places the folder tree among groups and falls back when its anchor disappears', () => {
	expect(sidebarSections(['a', 'b'], 'tree', undefined)).toEqual(['a', 'b', 'tree']);
	expect(sidebarSections(['a', 'b'], 'tree', null)).toEqual(['tree', 'a', 'b']);
	expect(sidebarSections(['a', 'b'], 'tree', 'a')).toEqual(['a', 'tree', 'b']);
	expect(sidebarSections(['a', 'b'], 'tree', 'removed')).toEqual(['a', 'b', 'tree']);
});


test('only sidebar sort handles start a keyboard drag', () => {
	expect(isSidebarSortable({label: 'Volume', sortable: {}})).toBe(true);
	expect(isSidebarSortable({label: 'File'})).toBe(false);
	expect(isSidebarSortable({sortable: {}})).toBe(false);
	expect(isSidebarSortable(undefined)).toBe(false);
});
