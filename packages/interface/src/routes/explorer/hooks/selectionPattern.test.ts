import {describe, expect, test} from 'bun:test';
import type {File} from '@wingdrive/ts-client';

import {filesMatching, globMatcher, invertSelection} from './selectionPattern';

const file = (id: string, name: string, extension: string | null = null) =>
	({id, name, extension}) as unknown as File;

const files = [
	file('1', 'IMG_001', 'JPG'),
	file('2', 'IMG_002', 'png'),
	file('3', 'notes', 'txt'),
	file('4', 'Photos'),
	file('5', 'a+b (1)', 'jpg')
];
const ids = (list: File[]) => list.map((f) => f.id);

describe('select by pattern', () => {
	test('matches whole names case-insensitively', () => {
		expect(ids(filesMatching(files, '*.jpg'))).toEqual(['1', '5']);
		expect(ids(filesMatching(files, 'photos'))).toEqual(['4']);
		expect(ids(filesMatching(files, 'IMG'))).toEqual([]);
	});

	test('supports ?, classes and several globs', () => {
		expect(ids(filesMatching(files, 'IMG_00?.*'))).toEqual(['1', '2']);
		expect(ids(filesMatching(files, 'IMG_00[2-9].*'))).toEqual(['2']);
		expect(ids(filesMatching(files, 'IMG_00[!1].*'))).toEqual(['2']);
		expect(ids(filesMatching(files, '*.png; *.txt'))).toEqual(['2', '3']);
	});

	test('treats regex characters literally', () => {
		expect(ids(filesMatching(files, 'a+b (1).jpg'))).toEqual(['5']);
		expect(globMatcher('[')?.test('[')).toBe(true);
	});

	test('an empty pattern selects nothing', () => {
		expect(globMatcher('  ')).toBeNull();
		expect(filesMatching(files, '')).toEqual([]);
	});
});

describe('invert selection', () => {
	test('selects every other listed file, in listing order', () => {
		expect(ids(invertSelection(files, [files[3]!, files[0]!]))).toEqual(['2', '3', '5']);
		expect(ids(invertSelection(files, []))).toEqual(['1', '2', '3', '4', '5']);
		expect(invertSelection(files, files)).toEqual([]);
	});
});
