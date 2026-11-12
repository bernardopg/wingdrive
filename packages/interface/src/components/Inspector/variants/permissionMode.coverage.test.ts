import {describe, expect, test} from 'bun:test';
import {formatMode, parseMode} from './permissionMode';

describe('permission mode boundaries', () => {
	test.each([
		['000', 0],
		['777', 0o777],
		['0000', 0],
		['7777', 0o7777],
		['  1755  ', 0o1755],
		['\t640\n', 0o640]
	])('parses %j as an octal mode', (input, expected) => {
		expect(parseMode(input)).toBe(expected);
	});
	test.each([
		'',
		' ',
		'77',
		'00000',
		'888',
		'-755',
		'+755',
		'0o755',
		'7 55',
		'７５５'
	])('rejects invalid input %j', (input) => {
		expect(parseMode(input)).toBeNull();
	});
	test.each([
		[0, '0000'],
		[0o600, '0600'],
		[0o100755, '0755'],
		[0o104755, '4755']
	])('formats mode %i without file type bits', (input, expected) => {
		expect(formatMode(input)).toBe(expected);
	});
});
