import {describe, expect, it} from 'bun:test';
import type {File} from '@wingdrive/ts-client';
import {mapDeviceToFile, mapLocationToFile} from '@wingdrive/ts-client';
import {
	reconcileSelectedFiles,
	sameFiles,
	selectionCapabilities
} from './fileCapabilities';

function file(id: string, path = `/home/me/${id}`): File {
	return {
		id,
		name: id,
		kind: 'File',
		extension: null,
		wing_path: {Physical: {device_slug: 'local', path}}
	} as unknown as File;
}

const location = mapLocationToFile({
	id: 'loc-1',
	name: 'Home',
	wing_path: {Physical: {device_slug: 'local', path: '/home/me'}}
});

describe('selectionCapabilities', () => {
	it('allows file commands on real entries', () => {
		const caps = selectionCapabilities([file('a'), file('b')]);
		expect(caps.canCopy).toBe(true);
		expect(caps.canDelete).toBe(true);
		expect(caps.canDuplicate).toBe(true);
		expect(caps.canRename).toBe(false);
	});

	it('allows rename for a single entry only', () => {
		expect(selectionCapabilities([file('a')]).canRename).toBe(true);
	});

	it('blocks every file command when a virtual entry is selected', () => {
		const caps = selectionCapabilities([file('a'), location]);
		expect(caps.hasVirtual).toBe(true);
		expect(caps.operable.map((f) => f.id)).toEqual(['a']);
		expect(caps.canCopy).toBe(false);
		expect(caps.canDelete).toBe(false);
		expect(caps.canRename).toBe(false);
		expect(caps.canDuplicate).toBe(false);
	});

	it('never exposes a device root as an operable target', () => {
		const device = mapDeviceToFile({id: 'd', slug: 'local', name: 'PC'});
		const caps = selectionCapabilities([device]);
		expect(caps.operable).toEqual([]);
		expect(caps.canDelete).toBe(false);
		expect(caps.canOpen).toBe(true);
	});

	it('does not duplicate entries without a local path', () => {
		const content = {
			...file('c'),
			wing_path: {Content: {content_id: 'x'}}
		} as unknown as File;
		expect(selectionCapabilities([content]).canDuplicate).toBe(false);
	});

	it('reports nothing for an empty selection', () => {
		const caps = selectionCapabilities([]);
		expect(caps.canCopy).toBe(false);
		expect(caps.canOpen).toBe(false);
	});
});

describe('reconcileSelectedFiles', () => {
	it('drops ids that are no longer displayed', () => {
		const result = reconcileSelectedFiles(
			['a', 'gone', 'b'],
			[file('a'), file('b'), file('c')]
		);
		expect(result.map((f) => f.id)).toEqual(['a', 'b']);
	});

	it('uses the latest object so a renamed path is not reused', () => {
		const renamed = file('a', '/home/me/renamed');
		const [result] = reconcileSelectedFiles(['a'], [renamed]);
		expect(result).toBe(renamed);
	});

	it('keeps the selection order', () => {
		const result = reconcileSelectedFiles(
			['c', 'a'],
			[file('a'), file('c')]
		);
		expect(result.map((f) => f.id)).toEqual(['c', 'a']);
	});

	it('returns nothing for an empty collection', () => {
		expect(reconcileSelectedFiles(['a'], [])).toEqual([]);
	});

	it('follows a file whose id was reconciled to its persistent uuid', () => {
		const ephemeral = file('temp', '/home/me/photo.jpg');
		const persistent = file('persistent', '/home/me/photo.jpg');
		const result = reconcileSelectedFiles(['temp'], [persistent], [ephemeral]);
		expect(result).toEqual([persistent]);
	});

	it('drops an unknown id even with a previous selection', () => {
		const result = reconcileSelectedFiles(['temp'], [file('other')], [file('x')]);
		expect(result).toEqual([]);
	});
});

describe('sameFiles', () => {
	it('compares identity and order', () => {
		const a = file('a');
		const b = file('b');
		expect(sameFiles([a, b], [a, b])).toBe(true);
		expect(sameFiles([a, b], [b, a])).toBe(false);
		expect(sameFiles([a], [file('a')])).toBe(false);
	});
});
