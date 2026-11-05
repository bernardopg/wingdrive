import {describe, expect, test} from 'bun:test';

import {buildJobNotification, shouldNotifyForJob} from './jobNotification';

const t = (key: string) => `t(${key})`;

describe('shouldNotifyForJob', () => {
	test('notifies for file operations while the window is in the background', () => {
		expect(shouldNotifyForJob('files.copy', false)).toBe(true);
		expect(shouldNotifyForJob('files.delete', false)).toBe(true);
	});

	test('stays quiet while the window is focused', () => {
		expect(shouldNotifyForJob('files.copy', true)).toBe(false);
	});

	test('stays quiet for background jobs without a user action', () => {
		expect(shouldNotifyForJob(undefined, false)).toBe(false);
		expect(shouldNotifyForJob(null, false)).toBe(false);
		expect(shouldNotifyForJob('locations.add', false)).toBe(false);
	});
});

describe('buildJobNotification', () => {
	test('a finished job is a transfer.complete notification', () => {
		expect(buildJobNotification('completed', "Copying 'a.txt'", t)).toEqual({
			title: 't(dialogs:jobs.notificationCompleted)',
			body: "Copying 'a.txt'",
			kind: 'transferComplete',
			openLabel: 't(dialogs:jobs.notificationOpen)'
		});
	});

	test('a failed job is a transfer.error notification', () => {
		const request = buildJobNotification('failed', "Moving 'b'", t);
		expect(request.title).toBe('t(dialogs:jobs.jobFailed)');
		expect(request.kind).toBe('transferError');
	});
});
