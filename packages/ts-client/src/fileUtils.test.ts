import {expect, test} from 'bun:test';
import type {File} from './generated/types';
import {fileDisplaySize, withDirectorySizes} from './fileUtils';

test('never presents directory metadata as a content total', () => {
	expect(
		fileDisplaySize({
			kind: 'Directory',
			size: 5052,
			directory_size: null
		})
	).toBeNull();
	expect(
		fileDisplaySize({
			kind: 'Directory',
			size: 5052,
			directory_size: 8192
		})
	).toBe(8192);
	expect(
		fileDisplaySize({kind: 'File', size: 5052, directory_size: null})
	).toBe(5052);
});

type Listed = Pick<File, 'kind' | 'wing_path' | 'directory_size'>;

function physical(kind: File['kind'], path: string, directory_size: number | null): Listed {
	return {
		kind,
		wing_path: {Physical: {device_slug: 'local', path}},
		directory_size
	};
}

test('measured directory totals replace unknown and stale stored totals', () => {
	const files = [
		physical('Directory', '/home/a', null),
		physical('Directory', '/home/b', 100),
		physical('Directory', '/home/c', 7),
		physical('File', '/home/d.txt', null)
	];

	const result = withDirectorySizes(files, {
		'/home/a': 10,
		'/home/b': 250,
		'/home/d.txt': 99
	});

	expect(result.map((file) => file.directory_size)).toEqual([10, 250, 7, null]);
	expect(result[2]).toBe(files[2]);
	expect(withDirectorySizes(files, undefined)).toBe(files);
});
