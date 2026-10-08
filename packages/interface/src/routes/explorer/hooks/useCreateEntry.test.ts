import {describe, expect, test} from 'bun:test';

import {nextFreeName} from './useCreateEntry';

describe('nextFreeName', () => {
	test('numbers before the extension and skips taken names', () => {
		expect(nextFreeName('Untitled File', [])).toBe('Untitled File');
		expect(nextFreeName('Untitled File', ['Untitled File', 'Untitled File 2'])).toBe(
			'Untitled File 3'
		);
		expect(nextFreeName('notes.txt (link)', ['notes.txt (link)'])).toBe('notes.txt (link) 2');
		expect(nextFreeName('a.tar', ['a.tar'])).toBe('a 2.tar');
		expect(nextFreeName('.bashrc', ['.bashrc'])).toBe('.bashrc 2');
	});
});
