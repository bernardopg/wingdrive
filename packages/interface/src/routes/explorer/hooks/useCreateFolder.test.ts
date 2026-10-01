import {describe, expect, it} from 'bun:test';
import {nextFolderName} from './useCreateFolder';

describe('nextFolderName', () => {
	it('uses the base name when free', () => {
		expect(nextFolderName(['a.txt'])).toBe('Untitled Folder');
	});

	it('skips taken numbered names', () => {
		expect(
			nextFolderName(['Untitled Folder', 'Untitled Folder 2', 'Untitled Folder 4'])
		).toBe('Untitled Folder 3');
	});
});
