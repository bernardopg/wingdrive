import {describe, expect, test} from 'bun:test';
import type {WingPath} from '@wingdrive/ts-client';

import {columnKey, folderKey, parentFolderKey} from './columnPaths';

const local = (path: string): WingPath => ({Physical: {device_slug: 'pc', path}});
const drive = (identifier: string, path: string) =>
	({Cloud: {service: 'gdrive', identifier, path}}) as unknown as WingPath;

describe('column path keys', () => {
	test('a file matches the column of its folder', () => {
		expect(parentFolderKey(local('/home/me/a.txt'))).toBe(folderKey(local('/home/me')));
		expect(parentFolderKey(drive('a@x.com', 'trip/a.jpg'))).toBe(folderKey(drive('a@x.com', 'trip')));
		expect(parentFolderKey(drive('a@x.com', 'a.jpg'))).toBe(folderKey(drive('a@x.com', '')));
	});

	test('cloud keys tell accounts apart', () => {
		expect(folderKey(drive('a@x.com', 'trip'))).not.toBe(folderKey(drive('b@x.com', 'trip')));
		expect(columnKey(drive('a@x.com', 'trip'), 1)).not.toBe(columnKey(drive('b@x.com', 'trip'), 1));
	});
});
