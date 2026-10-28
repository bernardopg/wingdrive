import {describe, expect, test} from 'bun:test';
import type {WingPath} from '@wingdrive/ts-client';

import {targetToUrl} from './context';
import {folderOfRoute, wingPathFromParam} from './wingPathParam';

const folder = (path: string): WingPath => ({Physical: {device_slug: 'arch', path}});

describe('wingPathFromParam', () => {
	test('reads a path whose name holds % and other escapes', () => {
		const path = folder('/home/me/100% café #1');
		expect(wingPathFromParam(JSON.stringify(path))).toEqual(path);
	});

	test('still reads a parameter encoded twice', () => {
		const path = folder('/tmp/a b');
		expect(wingPathFromParam(encodeURIComponent(JSON.stringify(path)))).toEqual(path);
	});

	test('rejects garbage', () => {
		expect(wingPathFromParam('not json')).toBeNull();
		expect(wingPathFromParam('%E0%A4%A')).toBeNull();
	});
});

describe('folderOfRoute', () => {
	test('round-trips the Explorer route of a folder', () => {
		const path = folder('/home/me/100%');
		expect(folderOfRoute(targetToUrl({type: 'path', path}))).toEqual(path);
	});

	test('is null for views and other routes', () => {
		expect(folderOfRoute(targetToUrl({type: 'view', view: 'device', id: 'x'}))).toBeNull();
		expect(folderOfRoute('/recents')).toBeNull();
		expect(folderOfRoute('/explorer?path=%7B%22Content%22%3A%7B%22content_id%22%3A%22c%22%7D%7D')).toBeNull();
	});
});
