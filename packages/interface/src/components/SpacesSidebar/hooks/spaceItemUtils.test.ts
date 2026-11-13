import {describe, expect, test} from 'bun:test';

import {volumeRootPath} from './spaceItemUtils';

describe('volume root path', () => {
	test('a disk volume opens its mount point on the device', () => {
		expect(volumeRootPath({device_slug: 'pc', mount_path: '/run/media/me/usb'})).toEqual({
			Physical: {device_slug: 'pc', path: '/run/media/me/usb'}
		});
		expect(volumeRootPath({device_slug: 'pc', mount_path: ''})).toEqual({
			Physical: {device_slug: 'pc', path: '/'}
		});
	});

	test('a cloud volume opens its cloud root', () => {
		expect(volumeRootPath({device_slug: 'pc', mount_path: 'gdrive://me@gmail.com'})).toEqual({
			Cloud: {service: 'gdrive', identifier: 'me@gmail.com', path: ''}
		} as never);
		expect(volumeRootPath({device_slug: 'pc', mount_path: 's3://bucket/prefix'})).toEqual({
			Cloud: {service: 's3', identifier: 'bucket', path: 'prefix'}
		} as never);
	});
});
