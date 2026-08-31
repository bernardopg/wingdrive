import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import {describe, expect, it} from 'bun:test';
import type {LibraryQuery, SourceInfo, SourceItem} from '../generated/types';

// Type-level fixture: outputs derived from the generated union must accept
// zero, one, and many records. The old one-element tuples (`[SourceInfo]`)
// rejected the zero and many cases, so tsc fails if the union regresses.
type SourceListOutput = Extract<LibraryQuery, {type: 'sources.list'}>['output'];
type SourceItemsOutput = Extract<
	LibraryQuery,
	{type: 'sources.list_items'}
>['output'];

const makeSource = (id: string): SourceInfo => ({
	id,
	name: id,
	data_type: 'test',
	adapter_id: 'test-adapter',
	item_count: 0,
	last_synced: null,
	status: 'ready'
});

const makeItem = (id: string): SourceItem => ({
	id,
	external_id: `ext-${id}`,
	title: id,
	preview: null,
	subtitle: null
});

const zeroSources: SourceListOutput = [];
const oneSource: SourceListOutput = [makeSource('one')];
const manySources: SourceListOutput = [
	makeSource('a'),
	makeSource('b'),
	makeSource('c')
];
const zeroItems: SourceItemsOutput = [];
const oneItem: SourceItemsOutput = [makeItem('one')];
const manyItems: SourceItemsOutput = [makeItem('a'), makeItem('b')];

describe('generated collection query types', () => {
	const generated = readFileSync(
		join(import.meta.dir, '../generated/types.ts'),
		'utf8'
	);

	it('accepts zero, one, and multiple records', () => {
		expect(zeroSources).toHaveLength(0);
		expect(oneSource).toHaveLength(1);
		expect(manySources).toHaveLength(3);
		expect(zeroItems).toHaveLength(0);
		expect(oneItem).toHaveLength(1);
		expect(manyItems).toHaveLength(2);
	});

	it('emits array types for collection queries', () => {
		for (const member of [
			'output: SourceInfo[]',
			'output: SourceItem[]',
			'output: LibraryInfo[]',
			'output: AdapterInfo[]',
			'output: AdapterConfigField[]',
			'output: Device[]'
		]) {
			expect(generated).toContain(member);
		}
	});

	it('never emits one-element tuple members', () => {
		// `[T]` in TypeScript is a tuple of exactly one element, not an array.
		expect(generated).not.toMatch(/(input|output): \[[A-Za-z][^\]]*\]/);
	});
});
