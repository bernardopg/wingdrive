import {describe, expect, test} from 'bun:test';
import type {File} from '@wingdrive/ts-client';

import {attachChildren, blocksByTopLevel, expandedWithin, toggleExpandedPath} from './listExpansion';

const entry = (path: string, kind = 'Directory') =>
	({id: path, kind, wing_path: {Physical: {device_slug: 'local', path}}}) as unknown as File;

describe('list expansion', () => {
	test('toggling collapses nested folders with their parent', () => {
		const opened = toggleExpandedPath(toggleExpandedPath([], '/a'), '/a/b');
		expect(opened).toEqual(['/a', '/a/b']);
		expect(toggleExpandedPath([...opened, '/ab'], '/a')).toEqual(['/ab']);
	});

	test('keeps only expansions inside the shown folder', () => {
		expect(expandedWithin(['/home/x', '/home/x/y', '/etc/z'], '/home')).toEqual(['/home/x', '/home/x/y']);
		expect(expandedWithin(['/home/x'], '/home/x')).toEqual([]);
		expect(expandedWithin(['/x'], null)).toEqual([]);
	});

	test('attaches listings level by level and skips files', () => {
		const top = [entry('/r/a'), entry('/r/b'), entry('/r/f.txt', 'File')];
		const {subRows, expanded} = attachChildren(
			top,
			['/r/a', '/r/a/c', '/r/f.txt', '/r/gone'],
			[[entry('/r/a/c'), entry('/r/a/d.txt', 'File')], [entry('/r/a/c/e', 'File')], [], []]
		);

		expect([...subRows.keys()]).toEqual(['/r/a', '/r/a/c']);
		expect(expanded).toEqual({'/r/a': true, '/r/a/c': true});
	});

	test('marks a folder expanded before its listing arrives', () => {
		const {subRows, expanded} = attachChildren([entry('/r/a')], ['/r/a'], [undefined]);
		expect(subRows.size).toBe(0);
		expect(expanded).toEqual({'/r/a': true});
	});

	test('groups descendants under their top-level row', () => {
		const rows = [
			{id: 'a', depth: 0},
			{id: 'a1', depth: 1},
			{id: 'a2', depth: 2},
			{id: 'b', depth: 0}
		];
		expect([...blocksByTopLevel(rows)].map(([id, block]) => [id, block.map((r) => r.id)])).toEqual([
			['a', ['a', 'a1', 'a2']],
			['b', ['b']]
		]);
	});
});
