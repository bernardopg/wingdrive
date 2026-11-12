import {describe, expect, test} from 'bun:test';
import type {File} from '@wingdrive/ts-client';

import {canPickDestination, pickedDestination, pickerStartFolder} from './copyMoveDestination';

const local = (path: string, is_local = true) =>
	({wing_path: {Physical: {device_slug: 'pc', path}}, is_local}) as unknown as File;
const cloud = {wing_path: {Cloud: {service: 'S3', identifier: 'b', path: 'x'}}, is_local: false} as unknown as File;

describe('Copy To / Move To destination', () => {
	test('uses the picked folder on the sources device', () => {
		expect(pickedDestination([local('/a/x')], '/b')).toEqual({Physical: {device_slug: 'pc', path: '/b'}});
		expect(pickedDestination([local('/a/x')], ['/c', '/d'])).toEqual({Physical: {device_slug: 'pc', path: '/c'}});
	});

	test('a cancelled picker does nothing', () => {
		expect(pickedDestination([local('/a/x')], null)).toBeNull();
		expect(pickedDestination([local('/a/x')], [])).toBeNull();
	});

	test('needs local or cloud sources', () => {
		expect(canPickDestination([])).toBe(false);
		expect(canPickDestination([local('/a', false)])).toBe(false);
		expect(canPickDestination([local('/a'), cloud])).toBe(true);
	});

	test('a cloud source downloads to a folder on this device', () => {
		expect(pickedDestination([cloud], '/b')).toEqual({Physical: {device_slug: 'local', path: '/b'}});
	});

	test('the picker starts in the folder that holds the selection', () => {
		expect(pickerStartFolder([local('/home/me/docs/a.txt'), local('/home/me/docs/b')])).toBe('/home/me/docs');
		expect(pickerStartFolder([local('/a.txt')])).toBe('/');
		expect(pickerStartFolder([local('C:\\Users\\me\\a.txt')])).toBe('C:\\Users\\me');
		expect(pickerStartFolder([local('C:\\a.txt')])).toBe('C:\\');
		expect(pickerStartFolder([cloud])).toBeUndefined();
		expect(pickerStartFolder([])).toBeUndefined();
	});
});
