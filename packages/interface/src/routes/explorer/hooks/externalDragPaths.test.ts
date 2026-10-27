import {describe, expect, it} from 'bun:test';
import type {File, WingPath} from '@wingdrive/ts-client';
import {mapLocationToFile} from '@wingdrive/ts-client';
import {externalDragPaths} from './externalDragPaths';

function file(id: string, wing_path: WingPath): File {
	return {id, name: id, kind: 'File', wing_path} as File;
}

const local = (path: string): WingPath => ({Physical: {device_slug: 'local', path}});

describe('externalDragPaths', () => {
	it('uses the whole selection only when the dragged file is selected', () => {
		const a = file('a', local('/tmp/a b'));
		const b = file('b', local('/tmp/é#b'));
		expect(externalDragPaths(a, [a, b])).toEqual(['/tmp/a b', '/tmp/é#b']);
		expect(externalDragPaths(a, [b])).toEqual(['/tmp/a b']);
	});

	it('accepts the current device slug and rejects remote and content paths', () => {
		const a = file('a', {Physical: {device_slug: 'device-a', path: '/tmp/a'}});
		const b = file('b', {Physical: {device_slug: 'device-b', path: '/tmp/b'}});
		const content = file('c', {Content: {content_id: 'c'}});
		const cloud = file('d', {Cloud: {service: 'S3', identifier: 'bucket', path: 'd'}});
		const location = mapLocationToFile({id: 'location', name: 'Home', wing_path: local('/home')});
		const missingPath = file('e', local(''));
		expect(externalDragPaths(a, [a], 'device-a')).toEqual(['/tmp/a']);
		expect(externalDragPaths(a, [a], undefined)).toEqual([]);
		expect(externalDragPaths(a, [a, b], 'device-a')).toEqual([]);
		expect(externalDragPaths(content, [content], 'device-a')).toEqual([]);
		expect(externalDragPaths(cloud, [cloud], 'device-a')).toEqual([]);
		expect(externalDragPaths(a, [a, location], 'device-a')).toEqual([]);
		expect(externalDragPaths(missingPath, [missingPath], 'device-a')).toEqual([]);
	});
});
