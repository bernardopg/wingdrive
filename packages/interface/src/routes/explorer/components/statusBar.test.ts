import {describe, expect, test} from 'bun:test';
import type {File, Volume} from '@wingdrive/ts-client';

import {summarizeFiles, volumeForPath} from './statusBar';

const entry = (kind: string, size: number, directory_size: number | null = null) =>
	({kind, size, directory_size}) as unknown as File;

const volume = (
	id: string,
	mount_point: string,
	device_id = 'dev',
	is_mounted = true,
	mount_points: string[] = []
) => ({id, mount_point, mount_points, device_id, is_mounted}) as unknown as Volume;

describe('summarizeFiles', () => {
	test('counts folders and files and sums known sizes', () => {
		expect(
			summarizeFiles([
				entry('File', 100),
				entry('Symlink', 5),
				entry('Directory', 0, 1000),
				entry('Directory', 0)
			])
		).toEqual({folders: 2, files: 2, bytes: 1105});
	});

	test('an empty list is all zeros', () => {
		expect(summarizeFiles([])).toEqual({folders: 0, files: 0, bytes: 0});
	});
});

describe('volumeForPath', () => {
	const volumes = [
		volume('root', '/'),
		volume('home', '/home'),
		volume('usb', '/run/media/me/USB'),
		volume('other', '/home', 'other-device'),
		volume('gone', '/mnt/old', 'dev', false),
		volume('alias', '/srv/data', 'dev', true, ['/data'])
	];
	const id = (path: string) => volumeForPath(volumes, 'dev', path)?.id ?? null;

	test('picks the longest mount point the path sits under', () => {
		expect(id('/home/me/Documents')).toBe('home');
		expect(id('/home')).toBe('home');
		expect(id('/etc')).toBe('root');
		expect(id('/run/media/me/USB/photos')).toBe('usb');
	});

	test('does not match a sibling that shares a prefix', () => {
		expect(id('/homework')).toBe('root');
	});

	test('skips other devices and unmounted volumes, reads extra mount points', () => {
		expect(volumeForPath(volumes, 'other-device', '/home/x')?.id).toBe('other');
		expect(id('/mnt/old/file')).toBe('root');
		expect(id('/data/file')).toBe('alias');
	});

	test('no match on an unknown device', () => {
		expect(volumeForPath(volumes, 'nope', '/home')).toBeNull();
	});
});
