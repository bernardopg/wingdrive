import {describe, expect, test} from 'bun:test';

import {formatMode, parseMode} from './permissionMode';

describe('octal mode', () => {
	test('round-trips and rejects invalid text', () => {
		expect(formatMode(0o755)).toBe('0755');
		expect(formatMode(0o4755)).toBe('4755');
		expect(parseMode('644')).toBe(0o644);
		expect(parseMode(' 0750 ')).toBe(0o750);
		expect(parseMode('789')).toBeNull();
		expect(parseMode('75')).toBeNull();
		expect(parseMode('17777')).toBeNull();
	});
});
