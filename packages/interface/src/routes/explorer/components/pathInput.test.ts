import {describe, expect, test} from 'bun:test';

import {resolveTypedPath} from './pathInput';

const cwd = '/home/me/Projects';

describe('resolveTypedPath', () => {
	test('expands ~ to the home folder', () => {
		expect(resolveTypedPath('~', '/home/me', cwd)).toBe('/home/me');
		expect(resolveTypedPath('~/', '/home/me', cwd)).toBe('/home/me');
		expect(resolveTypedPath(' ~/Docs/a b ', '/home/me/', cwd)).toBe('/home/me/Docs/a b');
		expect(resolveTypedPath('~/x', '/', cwd)).toBe('/x');
	});

	test('leaves ~ unresolved without a home folder', () => {
		expect(resolveTypedPath('~/Docs', null, cwd)).toBeNull();
	});

	test('reads names without a leading slash from the folder being shown', () => {
		expect(resolveTypedPath('Downloads', '/home/me', '/home/me')).toBe('/home/me/Downloads');
		expect(resolveTypedPath('src/lib', '/home/me', cwd)).toBe('/home/me/Projects/src/lib');
		expect(resolveTypedPath('~other/x', '/home/me', cwd)).toBe('/home/me/Projects/~other/x');
	});

	test('resolves . and .. segments', () => {
		expect(resolveTypedPath('..', '/home/me', cwd)).toBe('/home/me');
		expect(resolveTypedPath('../../..', '/home/me', cwd)).toBe('/');
		expect(resolveTypedPath('/etc/./ssh/../hosts', null, cwd)).toBe('/etc/hosts');
	});

	test('keeps absolute paths and ignores blank input', () => {
		expect(resolveTypedPath('/tmp/~', '/home/me', cwd)).toBe('/tmp/~');
		expect(resolveTypedPath('   ', '/home/me', cwd)).toBeNull();
	});
});
