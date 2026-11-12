import {describe, expect, test} from 'bun:test';
import {resolveTypedPath} from './pathInput';

describe('typed path edge cases', () => {
	test.each([
		['/', '/'],
		['////', '/'],
		['/a//b/', '/a/b'],
		['/a/../../b', '/b'],
		['./', '/home/me'],
		['../sibling', '/home/sibling'],
		['a/../b', '/home/me/b'],
		['a/.../b', '/home/me/a/.../b'],
		['~/../shared', '/home/shared'],
		['~someone', '/home/me/~someone'],
		['  /tmp/a b  ', '/tmp/a b'],
		['/tmp/.hidden', '/tmp/.hidden']
	])('resolves %j against the current folder', (input, expected) => {
		expect(resolveTypedPath(input, '/home/me', '/home/me')).toBe(expected);
	});
	test('does not expand a home path when home is unknown', () => {
		expect(resolveTypedPath('~', null, '/tmp')).toBeNull();
	});
	test('resolves a relative path from root', () => {
		expect(resolveTypedPath('child', null, '/')).toBe('/child');
	});
});
