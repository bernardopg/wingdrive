import {describe, expect, test} from 'bun:test';
import type {File} from '@wingdrive/ts-client';

import {canUploadToPhotos, isPhotosMedia} from './useGooglePhotosUpload';

const file = (name: string, extra: Partial<Record<string, unknown>> = {}) =>
	({
		name,
		kind: 'File',
		is_local: true,
		archive: null,
		wing_path: {Physical: {device_slug: 'local', path: `/home/u/${name}`}},
		...extra
	}) as unknown as File;

describe('Upload to Google Photos', () => {
	test('accepts photo and video extensions in any case', () => {
		expect(isPhotosMedia('IMG_0001.JPG')).toBe(true);
		expect(isPhotosMedia('clip.mov')).toBe(true);
		expect(isPhotosMedia('scan.heic')).toBe(true);
		expect(isPhotosMedia('notes.txt')).toBe(false);
		expect(isPhotosMedia('.jpg')).toBe(false);
		expect(isPhotosMedia('noextension')).toBe(false);
	});

	test('offers only local media and folders', () => {
		expect(canUploadToPhotos([file('a.jpg'), file('b.mp4')])).toBe(true);
		expect(canUploadToPhotos([file('Trip', {kind: 'Directory'})])).toBe(true);
		expect(canUploadToPhotos([file('a.jpg'), file('b.pdf')])).toBe(false);
		expect(canUploadToPhotos([])).toBe(false);
		expect(canUploadToPhotos([file('a.jpg', {is_local: false})])).toBe(false);
		expect(canUploadToPhotos([file('a.jpg', {archive: '/x.zip'})])).toBe(false);
		expect(
			canUploadToPhotos([
				file('a.jpg', {wing_path: {Cloud: {service: 'gdrive', identifier: 'me@x.com', path: 'a.jpg'}}})
			])
		).toBe(false);
	});
});
