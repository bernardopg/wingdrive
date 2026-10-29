import {describe, expect, mock, test} from 'bun:test';
import {Lightning} from '@phosphor-icons/react';

mock.module('@wingdrive/primitives', () => ({toast: {error: () => {}}}));

const {contextActionMenus} = await import('./useContextActions');

const action = (id: string, submenu: string | null = null) => ({
	id,
	name: id.toUpperCase(),
	icon: null,
	submenu
});

describe('contextActionMenus', () => {
	test('groups actions by their submenu, ungrouped ones under the fallback', () => {
		const ran: string[] = [];
		const menus = contextActionMenus(
			[action('a'), action('resize', 'Images'), action('b'), action('crop', 'Images')],
			'Actions',
			Lightning,
			(a) => ran.push(a.id)
		);

		expect(menus.map((menu) => [menu.label, menu.submenu?.map((item) => item.label)])).toEqual([
			['Actions', ['A', 'B']],
			['Images', ['RESIZE', 'CROP']]
		]);
		menus[1]?.submenu?.[1]?.onClick?.();
		expect(ran).toEqual(['crop']);
	});

	test('shows nothing without actions', () => {
		expect(contextActionMenus([], 'Actions', Lightning, () => {})).toEqual([]);
	});
});
