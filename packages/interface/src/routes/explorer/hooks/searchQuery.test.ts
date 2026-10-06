import {describe, expect, it} from 'bun:test';
import type {WingPath} from '@wingdrive/ts-client';
import {
	buildSearchInput,
	findLocationRoot,
	SEARCH_PAGE_SIZE,
	searchSortField,
	type SearchRequest
} from './searchQuery';

const path = (p: string, device_slug = 'vm'): WingPath => ({
	Physical: {device_slug, path: p}
});

const base: SearchRequest = {
	query: 'report',
	scope: 'folder',
	currentPath: path('/home/me/docs'),
	locationRoot: path('/home/me'),
	sortBy: 'name',
	sortDirection: 'Asc',
	includeHidden: false,
	contentTypes: [],
	page: 0
};

describe('buildSearchInput', () => {
	it('searches under the current folder', () => {
		expect(buildSearchInput(base).scope).toEqual({
			Path: {path: path('/home/me/docs')}
		});
	});

	it('searches under the location root for location scope', () => {
		expect(buildSearchInput({...base, scope: 'location'}).scope).toEqual({
			Path: {path: path('/home/me')}
		});
	});

	it('uses the library for library scope', () => {
		expect(buildSearchInput({...base, scope: 'library'}).scope).toBe(
			'Library'
		);
	});

	it('honors the sort direction', () => {
		expect(buildSearchInput({...base, sortDirection: 'Desc'}).sort).toEqual({
			field: 'Name',
			direction: 'Desc'
		});
	});

	it('passes hidden files and content kinds through', () => {
		const input = buildSearchInput({
			...base,
			includeHidden: true,
			contentTypes: ['image', 'video']
		});
		expect(input.filters.include_hidden).toBe(true);
		expect(input.filters.content_types).toEqual(['image', 'video']);
	});

	it('pages with one extra row to detect the next page', () => {
		expect(buildSearchInput({...base, page: 2}).pagination).toEqual({
			limit: SEARCH_PAGE_SIZE + 1,
			offset: 2 * SEARCH_PAGE_SIZE
		});
	});
});

describe('searchSortField', () => {
	it('maps explorer sorts', () => {
		expect(searchSortField('modified')).toBe('ModifiedAt');
		expect(searchSortField('size')).toBe('Size');
		expect(searchSortField('type')).toBe('Relevance');
	});
});

describe('findLocationRoot', () => {
	const locations = [
		{wing_path: path('/home/me')},
		{wing_path: path('/home/me/photos')},
		{wing_path: path('/home/me', 'other')}
	];

	it('picks the deepest containing location', () => {
		expect(findLocationRoot(path('/home/me/photos/2026'), locations)).toEqual(
			path('/home/me/photos')
		);
	});

	it('matches whole path segments only', () => {
		expect(findLocationRoot(path('/home/me/photos2'), locations)).toEqual(
			path('/home/me')
		);
	});

	it('ignores locations on other devices', () => {
		expect(findLocationRoot(path('/srv'), locations)).toBeNull();
	});

	it('accepts the root itself', () => {
		expect(findLocationRoot(path('/home/me'), locations)).toEqual(
			path('/home/me')
		);
	});
});
