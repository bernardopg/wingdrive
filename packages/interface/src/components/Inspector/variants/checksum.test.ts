import {describe, expect, test} from 'bun:test';

import {compareSha256, parseExpectedSha256} from './checksum';

const abc = 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad';

describe('expected SHA-256 input', () => {
	test('accepts bare, prefixed and sha256sum forms', () => {
		expect(parseExpectedSha256(abc.toUpperCase())).toBe(abc);
		expect(parseExpectedSha256(`  sha256:${abc} `)).toBe(abc);
		expect(parseExpectedSha256(`${abc}  ./WingDrive.AppImage`)).toBe(abc);
	});

	test('rejects text without a full digest', () => {
		expect(parseExpectedSha256('')).toBeNull();
		expect(parseExpectedSha256(abc.slice(1))).toBeNull();
		expect(parseExpectedSha256(`${abc}0`)).toBeNull();
	});

	test('compares case-insensitively', () => {
		expect(compareSha256(abc, abc.toUpperCase())).toBe('match');
		expect(compareSha256(abc, 'e'.repeat(64))).toBe('mismatch');
		expect(compareSha256(abc, 'not a hash')).toBe('invalid');
	});
});
