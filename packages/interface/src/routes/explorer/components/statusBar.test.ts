import {describe, expect, test} from 'bun:test';
import type {Device, File, Volume} from '@wingdrive/ts-client';

import {deviceIdForSlug, summarizeFiles, volumeForPath} from './statusBar';

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

	test('matches Windows drives and folder mount points', () => {
		const windows = [volume('c', 'C:\\'), volume('data', 'C:\\mnt\\data'), volume('d', 'D:\\')];
		const winId = (path: string) => volumeForPath(windows, 'dev', path)?.id ?? null;
		expect(winId('C:\\Users\\me')).toBe('c');
		expect(winId('C:\\')).toBe('c');
		expect(winId('C:\\mnt\\data\\photos')).toBe('data');
		expect(winId('C:\\mnt\\database')).toBe('c');
		expect(winId('D:\\games')).toBe('d');
		expect(winId('E:\\x')).toBeNull();
	});

	test('no match on an unknown device', () => {
		expect(volumeForPath(volumes, 'nope', '/home')).toBeNull();
	});
});

describe('deviceIdForSlug', () => {
	const devices = [
		{id: 'phone', slug: 'pixel', is_current: false},
		{id: 'pc', slug: 'archlinux', is_current: true}
	] as unknown as Device[];

	test('matches the slug, and `local` names the current device', () => {
		expect(deviceIdForSlug(devices, 'pixel')).toBe('phone');
		expect(deviceIdForSlug(devices, 'archlinux')).toBe('pc');
		expect(deviceIdForSlug(devices, 'local')).toBe('pc');
		expect(deviceIdForSlug(devices, 'nope')).toBeNull();
		expect(deviceIdForSlug(devices.slice(0, 1), 'local')).toBeNull();
	});
});
