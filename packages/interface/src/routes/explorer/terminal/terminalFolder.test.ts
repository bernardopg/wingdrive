import {describe, expect, test} from 'bun:test';
import type {WingPath} from '@wingdrive/ts-client';

import {clampTerminalHeight, localFolder, MIN_TERMINAL_HEIGHT} from './terminalFolder';

const physical = (device_slug: string, path: string) =>
	({Physical: {device_slug, path}}) as WingPath;

describe('localFolder', () => {
	test('a folder on this device', () => {
		expect(localFolder(physical('local', '/home/me'), undefined)).toBe('/home/me');
		expect(localFolder(physical('desk', '/home/me'), 'desk')).toBe('/home/me');
	});

	test('nothing for another device, a view without a folder, or a relative path', () => {
		expect(localFolder(physical('phone', '/sdcard'), 'desk')).toBeNull();
		expect(localFolder(null, 'desk')).toBeNull();
		expect(localFolder({Content: {content_id: 'c'}} as unknown as WingPath, 'desk')).toBeNull();
		expect(localFolder(physical('local', 'relative'), 'desk')).toBeNull();
	});
});

describe('clampTerminalHeight', () => {
	test('keeps the panel between its minimum and most of the view', () => {
		expect(clampTerminalHeight(10, 1000)).toBe(MIN_TERMINAL_HEIGHT);
		expect(clampTerminalHeight(300.4, 1000)).toBe(300);
		expect(clampTerminalHeight(5000, 1000)).toBe(800);
	});

	test('a view too small for the maximum still gets the minimum', () => {
		expect(clampTerminalHeight(500, 50)).toBe(MIN_TERMINAL_HEIGHT);
	});
});
