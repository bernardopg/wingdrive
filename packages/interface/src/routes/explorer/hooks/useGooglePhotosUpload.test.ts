import {describe, expect, test} from 'bun:test';
import type {File} from '@wingdrive/ts-client';

import {canUploadToPhotos, isPhotosMedia} from './useGooglePhotosUpload';

// Like the daemon: `name` is the stem and the extension is a separate field.
const file = (fullName: string, extra: Partial<Record<string, unknown>> = {}) => {
	const dot = fullName.lastIndexOf('.');
	const hasExtension = dot > 0 && extra.kind !== 'Directory';
	return {
		name: hasExtension ? fullName.slice(0, dot) : fullName,
		extension: hasExtension ? fullName.slice(dot + 1) : null,
		kind: 'File',
		is_local: true,
		archive: null,
		wing_path: {Physical: {device_slug: 'local', path: `/home/u/${fullName}`}},
		...extra
	} as unknown as File;
};

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
