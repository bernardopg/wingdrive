import {QueryClient} from '@tanstack/react-query';
import {describe, expect, it} from 'bun:test';

import type {Event} from '../../generated/types';
import {
	deleteResource,
	filterBatchResources,
	handleResourceEvent,
	reconcileAlternateIds,
	safeMerge,
	updateBatchResources,
	updateSingleResource,
} from '../useNormalizedQuery';

function makeQueryClient() {
	return new QueryClient({
		defaultOptions: {queries: {retry: false}},
	});
}

describe('safeMerge', () => {
	it('replaces arrays and deep merges nested objects', () => {
		const merged = safeMerge(
			{id: '1', tags: ['a'], meta: {a: 1, b: 2}, name: 'old'},
			{id: '1', tags: ['b'], meta: {b: 3, c: 4}, name: 'new'},
		);
		expect(merged).toEqual({
			id: '1',
			tags: ['b'],
			meta: {a: 1, b: 3, c: 4},
			name: 'new',
		});
	});

	it('respects noMergeFields and null/undefined incoming values', () => {
		const merged = safeMerge(
			{id: '1', status: 'ready', count: 5},
			{id: '1', status: null, count: 6},
			['status'],
		);
		expect(merged.status).toBeNull();
		expect(merged.count).toBe(6);

		expect(safeMerge({a: 1}, null as any)).toEqual({a: 1});
		expect(safeMerge(null as any, {a: 1})).toEqual({a: 1});
		expect(safeMerge({a: 1}, undefined as any)).toEqual({a: 1});
	});
});

describe('filterBatchResources', () => {
	it('filters by resourceId', () => {
		const result = filterBatchResources([{id: 'a'}, {id: 'b'}], {
			query: 'q',
			input: {},
			resourceType: 'file',
			resourceId: 'b',
		});
		expect(result).toEqual([{id: 'b'}]);
	});

	it('filters exact-mode file batches to direct children of pathScope', () => {
		const options = {
			query: 'files.directory_listing',
			input: {},
			resourceType: 'file',
			pathScope: {Physical: {path: '/root/dir'}} as any,
			includeDescendants: false,
		};
		const resources = [
			{id: '1', wing_path: {Physical: {path: '/root/dir/a.txt'}}},
			{id: '2', wing_path: {Physical: {path: '/root/dir/sub/b.txt'}}},
			{id: '3', alternate_paths: [{Physical: {path: '/root/dir/c.txt'}}]},
			{id: '4', wing_path: {Content: {hash: 'x'}}},
		];
		expect(filterBatchResources(resources, options).map((r) => r.id)).toEqual([
			'1',
			'3',
		]);
	});

	it('handles windows-style scopes case-insensitively', () => {
		const options = {
			query: 'q',
			input: {},
			resourceType: 'file',
			pathScope: {Physical: {path: 'C:\\Users\\Test\\'}} as any,
			includeDescendants: false,
		};
		const resources = [
			{id: '1', wing_path: {Physical: {path: 'C:/Users/Test/file.txt'}}},
			{id: '2', wing_path: {Physical: {path: 'C:/Users/Test/Sub/file.txt'}}},
		];
		expect(filterBatchResources(resources, options).map((r) => r.id)).toEqual(['1']);
	});
});

describe('deleteResource', () => {
	it('removes from array and wrapped caches', () => {
		const qc = makeQueryClient();
		const arrayKey = ['k1'];
		const wrappedKey = ['k2'];
		qc.setQueryData(arrayKey, [{id: 'a'}, {id: 'b'}]);
		qc.setQueryData(wrappedKey, {files: [{id: 'a'}, {id: 'b'}], total_count: 2});

		deleteResource('a', arrayKey, qc);
		deleteResource('b', wrappedKey, qc);

		expect(qc.getQueryData(arrayKey)).toEqual([{id: 'b'}]);
		expect(qc.getQueryData(wrappedKey)).toEqual({files: [{id: 'a'}], total_count: 2});
	});
});

describe('reconcileAlternateIds', () => {
	it('replaces aliased entries and dedupes', () => {
		const oldData = [{id: 'old', name: 'Old'}, {id: 'keep', name: 'Keep'}];
		const result = reconcileAlternateIds(
			oldData,
			{id: 'new', name: 'New'},
			['old'],
			[],
		);
		expect(result).toEqual([
			{id: 'new', name: 'New'},
			{id: 'keep', name: 'Keep'},
		]);
	});

	it('merges directly when oldData itself is the alias', () => {
		const result = reconcileAlternateIds(
			{id: 'old', a: 1},
			{id: 'new', b: 2},
			['old'],
			[],
		);
		expect(result).toEqual({id: 'new', a: 1, b: 2});
	});
});

describe('updateSingleResource / updateBatchResources', () => {
	it('updates existing items and appends new items in wrapped cache', () => {
		const qc = makeQueryClient();
		const key = ['files'];
		qc.setQueryData(key, {files: [{id: '1', name: 'old'}]});

		updateBatchResources(
			[{id: '1', name: 'new'}, {id: '2', name: 'two'}],
			{},
			{query: 'files.list', input: {}, resourceType: 'file'},
			key,
			qc,
		);

		expect(qc.getQueryData(key)).toEqual({
			files: [
				{id: '1', name: 'new'},
				{id: '2', name: 'two'},
			],
		});
	});

	it('removes items that moved out of exact scope', () => {
		const qc = makeQueryClient();
		const key = ['files'];
		qc.setQueryData(key, {files: [{id: '1'}, {id: '2'}]});
		const options = {
			query: 'files.directory_listing',
			input: {},
			resourceType: 'file',
			pathScope: {Physical: {path: '/root'}} as any,
			includeDescendants: false,
		};

		updateBatchResources(
			[{id: '1', wing_path: {Physical: {path: '/other/1.txt'}}}],
			{},
			options,
			key,
			qc,
		);
		expect(qc.getQueryData(key)).toEqual({files: [{id: '2'}]});
	});

	it('seeds file query cache when no old data exists', () => {
		const qc = makeQueryClient();
		const key = ['files'];
		updateBatchResources(
			[{id: '1'}],
			{},
			{query: 'files.list', input: {}, resourceType: 'file'},
			key,
			qc,
		);
		expect(qc.getQueryData(key)).toEqual({
			files: [{id: '1'}],
			total_count: 1,
			has_more: false,
		});
	});

	it('does not seed non-file caches', () => {
		const qc = makeQueryClient();
		const key = ['devices'];
		updateBatchResources(
			[{id: '1'}],
			{},
			{query: 'devices.list', input: {}, resourceType: 'device'},
			key,
			qc,
		);
		expect(qc.getQueryData(key)).toBeUndefined();
	});

	it('updateSingleResource merges by id in wrapped cache', () => {
		const qc = makeQueryClient();
		const key = ['files'];
		qc.setQueryData(key, {files: [{id: '1', name: 'old'}]});

		updateSingleResource({id: '1', name: 'new'}, {}, key, qc, {
			query: 'files.list',
			input: {},
			resourceType: 'file',
		});
		expect(qc.getQueryData(key)).toEqual({files: [{id: '1', name: 'new'}]});
	});
});

describe('handleResourceEvent', () => {
	it('ignores string events', () => {
		const qc = makeQueryClient();
		const key = ['k'];
		qc.setQueryData(key, {files: []});
		handleResourceEvent(
			'CoreStarted' as any,
			{query: 'files.list', input: {}, resourceType: 'file'},
			key,
			qc,
		);
		expect(qc.getQueryData(key)).toEqual({files: []});
	});

	it('invalidates all queries on Refresh', () => {
		const qc = makeQueryClient();
		const key = ['k'];
		qc.setQueryData(key, {files: [{id: '1'}]});
		handleResourceEvent(
			{Refresh: null} as any,
			{query: 'files.list', input: {}, resourceType: 'file'},
			key,
			qc,
		);
		expect(qc.getQueryState(key)?.isInvalidated).toBe(true);
	});

	it('deletes resources on ResourceDeleted', () => {
		const qc = makeQueryClient();
		const key = ['k'];
		qc.setQueryData(key, {files: [{id: '1'}, {id: '2'}]});
		handleResourceEvent(
			{
				ResourceDeleted: {resource_type: 'file', resource_id: '1'},
			} as any,
			{query: 'files.list', input: {}, resourceType: 'file'},
			key,
			qc,
		);
		expect(qc.getQueryData(key)).toEqual({files: [{id: '2'}]});
	});
});
