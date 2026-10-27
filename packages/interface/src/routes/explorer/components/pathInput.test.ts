import {describe, expect, test} from 'bun:test';

import {resolveTypedPath} from './pathInput';

describe('resolveTypedPath', () => {
	test('expands ~ to the home folder', () => {
		expect(resolveTypedPath('~', '/home/me')).toBe('/home/me');
		expect(resolveTypedPath('~/', '/home/me')).toBe('/home/me');
		expect(resolveTypedPath(' ~/Docs/a b ', '/home/me/')).toBe('/home/me/Docs/a b');
		expect(resolveTypedPath('~/x', '/')).toBe('/x');
	});

	test('leaves ~ unresolved without a home folder', () => {
		expect(resolveTypedPath('~/Docs', null)).toBeNull();
	});

	test('does not expand ~user or a ~ inside a name', () => {
		expect(resolveTypedPath('~other/x', '/home/me')).toBe('/~other/x');
		expect(resolveTypedPath('/tmp/~', '/home/me')).toBe('/tmp/~');
	});

	test('reads relative paths from the root and ignores blank input', () => {
		expect(resolveTypedPath('etc', '/home/me')).toBe('/etc');
		expect(resolveTypedPath('   ', '/home/me')).toBeNull();
	});
});
