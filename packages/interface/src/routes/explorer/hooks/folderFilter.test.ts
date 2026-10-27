import {describe, expect, test} from 'bun:test';
import type {File} from '@wingdrive/ts-client';

import {filterFiles} from './folderFilter';

const file = (id: string, name: string, extension: string | null = null) =>
	({id, name, extension}) as unknown as File;

const files = [
	file('1', 'Report 2026', 'PDF'),
	file('2', 'report-draft', 'docx'),
	file('3', 'notes', 'txt'),
	file('4', 'Reports')
];
const ids = (list: File[]) => list.map((f) => f.id);

describe('folder filter', () => {
	test('blank text keeps every file', () => {
		expect(filterFiles(files, '')).toBe(files);
		expect(filterFiles(files, '   ')).toBe(files);
	});

	test('plain text is a case-insensitive substring of the full name', () => {
		expect(ids(filterFiles(files, 'REPORT'))).toEqual(['1', '2', '4']);
		expect(ids(filterFiles(files, '.pdf'))).toEqual(['1']);
		expect(ids(filterFiles(files, ' notes '))).toEqual(['3']);
		expect(ids(filterFiles(files, 'missing'))).toEqual([]);
	});

	test('glob characters match the whole name', () => {
		expect(ids(filterFiles(files, '*.pdf'))).toEqual(['1']);
		expect(ids(filterFiles(files, 'report*'))).toEqual(['1', '2', '4']);
		expect(ids(filterFiles(files, '*.pdf; *.txt'))).toEqual(['1', '3']);
	});
});
