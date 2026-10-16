import {describe, expect, test} from 'bun:test';
import type {File} from '@wingdrive/ts-client';

import {canPickDestination, pickedDestination} from './copyMoveDestination';

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

	test('needs local physical sources', () => {
		expect(canPickDestination([])).toBe(false);
		expect(canPickDestination([local('/a'), cloud])).toBe(false);
		expect(canPickDestination([local('/a', false)])).toBe(false);
		expect(pickedDestination([cloud], '/b')).toBeNull();
	});
});
