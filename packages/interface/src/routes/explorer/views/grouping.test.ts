import {describe, expect, test} from 'bun:test';
import type {File} from '@wingdrive/ts-client';

import {buildGridLayout, buildListLayout, dateLabel, gridNeighbor, groupFiles} from './grouping';

const NOW = new Date(2026, 9, 28, 15, 0);

const file = (overrides: Record<string, unknown>) =>
	({
		id: String(overrides.name),
		kind: 'File',
		extension: null,
		size: 0,
		directory_size: null,
		modified_at: NOW.toISOString(),
		created_at: NOW.toISOString(),
		tags: [],
		content_kind: 'unknown',
		...overrides
	}) as unknown as File;

const keysAndNames = (groups: ReturnType<typeof groupFiles>) =>
	groups.map((group) => [group.key, group.files.map((f) => f.name)]);

describe('groupFiles', () => {
	test('keeps the listing order and orders sections by their first file', () => {
		const files = [
			file({name: 'beta'}),
			file({name: 'alpha'}),
			file({name: 'banana'}),
			file({name: '42'})
		];

		expect(keysAndNames(groupFiles(files, 'name', NOW))).toEqual([
			['letter:B', ['beta', 'banana']],
			['letter:A', ['alpha']],
			['other', ['42']]
		]);
	});

	test('groups by type with folders first, then kinds alphabetically', () => {
		const files = [
			file({name: 'song', content_kind: 'audio'}),
			file({name: 'docs', kind: 'Directory'}),
			file({name: 'clip', content_kind: 'video'}),
			file({name: 'track', content_kind: 'audio'})
		];

		expect(keysAndNames(groupFiles(files, 'type', NOW))).toEqual([
			['folders', ['docs']],
			['kind:audio', ['song', 'track']],
			['kind:video', ['clip']]
		]);
	});

	test('groups by size with the search presets, unmeasured folders apart', () => {
		const files = [
			file({name: 'big', size: 2 * 1024 ** 3}),
			file({name: 'dir', kind: 'Directory'}),
			file({name: 'note', size: 200})
		];

		expect(keysAndNames(groupFiles(files, 'size', NOW))).toEqual([
			['size:huge', ['big']],
			['folders', ['dir']],
			['size:tiny', ['note']]
		]);
	});

	test('groups by the first tag', () => {
		const files = [
			file({name: 'a', tags: [{canonical_name: 'work'}]}),
			file({name: 'b'})
		];

		expect(keysAndNames(groupFiles(files, 'tags', NOW))).toEqual([
			['tag:work', ['a']],
			['untagged', ['b']]
		]);
	});
});

describe('dateLabel', () => {
	const at = (...parts: [number, number, number, number?]) =>
		new Date(parts[0], parts[1], parts[2], parts[3] ?? 12).toISOString();

	test('buckets recent dates relative to the start of today', () => {
		expect(dateLabel(at(2026, 9, 28, 1), NOW)).toEqual({type: 'date', bucket: 'today'});
		expect(dateLabel(at(2026, 9, 27, 23), NOW)).toEqual({type: 'date', bucket: 'yesterday'});
		expect(dateLabel(at(2026, 9, 22), NOW)).toEqual({type: 'date', bucket: 'week'});
		expect(dateLabel(at(2026, 9, 1), NOW)).toEqual({type: 'date', bucket: 'month'});
	});

	test('falls back to months of this year, then years', () => {
		expect(dateLabel(at(2026, 2, 3), NOW)).toEqual({type: 'month', year: 2026, month: 2});
		expect(dateLabel(at(2024, 6, 1), NOW)).toEqual({type: 'year', year: 2024});
	});

	test('treats future and missing dates sensibly', () => {
		expect(dateLabel(at(2026, 10, 2), NOW)).toEqual({type: 'date', bucket: 'today'});
		expect(dateLabel(null, NOW)).toEqual({type: 'other'});
		expect(dateLabel('not a date', NOW)).toEqual({type: 'other'});
	});
});

describe('grid and list layouts', () => {
	const files = ['a1', 'a2', 'a3', 'b1', 'b2'].map((name) => file({name}));
	const groups = groupFiles(files, 'name', NOW);

	test('starts each section on a new grid row', () => {
		const layout = buildGridLayout(groups, files, 2);

		expect(
			layout.rows.map((row) =>
				row.type === 'header' ? row.group.key : row.files.map((f) => f.name)
			)
		).toEqual(['letter:A', ['a1', 'a2'], ['a3'], 'letter:B', ['b1', 'b2']]);
		expect(layout.rowOfFile).toEqual([1, 1, 2, 4, 4]);
	});

	test('fills rows without headers when ungrouped', () => {
		const layout = buildGridLayout(null, files, 2);

		expect(layout.rows.map((row) => row.type === 'files' && row.start)).toEqual([0, 2, 4]);
	});

	test('moves vertically within the column across section headers', () => {
		const layout = buildGridLayout(groups, files, 2);

		expect(gridNeighbor(layout, 1, 'down')).toBe(2); // a2 to a3, the shorter row
		expect(gridNeighbor(layout, 2, 'down')).toBe(3); // a3 skips the header to b1
		expect(gridNeighbor(layout, 4, 'up')).toBe(2); // b2 to a3, clamped
		expect(gridNeighbor(layout, 0, 'up')).toBe(0);
	});

	test('interleaves list headers and maps files to items', () => {
		const layout = buildListLayout(groups, files.length);

		expect(layout.items.map((item) => (item.type === 'header' ? item.group.key : item.index))).toEqual([
			'letter:A',
			0,
			1,
			2,
			'letter:B',
			3,
			4
		]);
		expect(layout.itemOfFile).toEqual([1, 2, 3, 5, 6]);
		expect(buildListLayout(null, 2).itemOfFile).toEqual([0, 1]);
	});
});
