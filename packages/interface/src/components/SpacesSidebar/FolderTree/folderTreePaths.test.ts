import {describe, expect, test} from 'bun:test';

import {ancestorsWithin, isWithin, normalizeFolder, rootFor} from './folderTreePaths';

describe('folder tree paths', () => {
	test('normalizes trailing slashes but keeps the root', () => {
		expect(normalizeFolder('/home/me/')).toBe('/home/me');
		expect(normalizeFolder('/')).toBe('/');
		expect(normalizeFolder('//')).toBe('/');
	});

	test('checks containment by whole path components', () => {
		expect(isWithin('/home/me/Docs', '/home/me')).toBe(true);
		expect(isWithin('/home/me', '/home/me/')).toBe(true);
		expect(isWithin('/home/meow', '/home/me')).toBe(false);
		expect(isWithin('/etc', '/')).toBe(true);
	});

	test('lists the folders to expand down to a path', () => {
		expect(ancestorsWithin('/home/me/Docs/2026', '/home/me')).toEqual([
			'/home/me',
			'/home/me/Docs'
		]);
		expect(ancestorsWithin('/etc/systemd', '/')).toEqual(['/', '/etc']);
		expect(ancestorsWithin('/home/me', '/home/me')).toEqual([]);
		expect(ancestorsWithin('/var', '/home/me')).toEqual([]);
	});

	test('picks the deepest root containing a path', () => {
		const roots = ['/', '/home/me'];
		expect(rootFor('/home/me/Docs', roots)).toBe('/home/me');
		expect(rootFor('/etc', roots)).toBe('/');
		expect(rootFor('/etc', ['/home/me'])).toBeNull();
	});
});
