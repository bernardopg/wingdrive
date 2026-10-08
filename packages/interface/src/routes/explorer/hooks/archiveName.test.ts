import {describe, expect, test} from 'bun:test';

import {archiveBaseName, isArchiveName} from './archiveName';

describe('archive names', () => {
	test('detects archives by suffix', () => {
		expect(isArchiveName('photos.TAR.GZ')).toBe(true);
		expect(isArchiveName('a.7z')).toBe(true);
		expect(isArchiveName('notes.txt')).toBe(false);
		expect(isArchiveName('.zip')).toBe(false);
	});

	test('names archives after a single item', () => {
		expect(archiveBaseName(['report.pdf'])).toBe('report');
		expect(archiveBaseName(['Photos'])).toBe('Photos');
		expect(archiveBaseName(['a', 'b'])).toBe('Archive');
	});
});
