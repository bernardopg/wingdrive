import {describe, expect, test} from 'bun:test';

import {isPermissionError} from './adminRetry';

describe('isPermissionError', () => {
	test('recognizes the ways a permission failure reaches the UI', () => {
		for (const message of [
			'/etc/hosts: Permission denied',
			'Failed to create /opt/x: Permission denied (os error 13)',
			'Operation not permitted (os error 1)',
			'EACCES: open failed'
		]) {
			expect(isPermissionError(message)).toBe(true);
		}
	});

	test('ignores other failures', () => {
		for (const message of ['No such file or directory (os error 2)', 'os error 10', 'Disk full']) {
			expect(isPermissionError(message)).toBe(false);
		}
	});
});
