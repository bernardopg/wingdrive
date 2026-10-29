import type {File} from '@wingdrive/ts-client';
import {fileDisplaySize, getContentKind} from '@wingdrive/ts-client';

import type {SearchSizePreset} from '../context';
import {SEARCH_SIZE_PRESETS} from '../hooks/searchQuery';

/** Sort keys a folder can be grouped by: the Explorer's and the list's columns. */
export type GroupBy = 'name' | 'modified' | 'created' | 'size' | 'type' | 'tags';

export type DateBucket = 'today' | 'yesterday' | 'week' | 'month';

/** What a section header says, before translation. */
export type GroupLabel =
	| {type: 'folders'}
	| {type: 'letter'; letter: string}
	| {type: 'other'}
	| {type: 'date'; bucket: DateBucket}
	| {type: 'month'; year: number; month: number}
	| {type: 'year'; year: number}
	| {type: 'size'; preset: SearchSizePreset}
	| {type: 'kind'; kind: string}
	| {type: 'tag'; name: string}
	| {type: 'untagged'};

export interface FileGroup {
	key: string;
	label: GroupLabel;
	files: File[];
}

const DAY_MS = 24 * 60 * 60 * 1000;

export function isGroupBy(value: string | undefined): value is GroupBy {
	return (
		value === 'name' ||
		value === 'modified' ||
		value === 'created' ||
		value === 'size' ||
		value === 'type' ||
		value === 'tags'
	);
}

/**
 * Splits an ordered listing into labelled sections.
 *
 * Files keep their order inside a section, and sections appear in the order
 * their first file does, so the active sort direction carries over. Grouping
 * by type is the exception: folders come first, then kinds alphabetically,
 * because the type sort orders by name, not by kind.
 */
export function groupFiles(files: File[], by: GroupBy, now = new Date()): FileGroup[] {
	const groups = new Map<string, FileGroup>();

	for (const file of files) {
		const label = groupLabel(file, by, now);
		const key = groupKey(label);
		let group = groups.get(key);
		if (!group) {
			group = {key, label, files: []};
			groups.set(key, group);
		}
		group.files.push(file);
	}

	const ordered = [...groups.values()];
	if (by === 'type') {
		ordered.sort((a, b) => {
			if (a.label.type === 'folders') return -1;
			if (b.label.type === 'folders') return 1;
			return a.key.localeCompare(b.key);
		});
	}
	return ordered;
}

export function groupLabel(file: File, by: GroupBy, now: Date): GroupLabel {
	switch (by) {
		case 'name': {
			const letter = fileName(file).charAt(0).toLocaleUpperCase();
			return /\p{L}/u.test(letter) ? {type: 'letter', letter} : {type: 'other'};
		}
		case 'modified':
			return dateLabel(file.modified_at, now);
		case 'created':
			return dateLabel(file.created_at, now);
		case 'size': {
			const size = fileDisplaySize(file);
			if (size === null) return {type: 'folders'};
			const preset = SEARCH_SIZE_PRESETS.find(
				({min, max}) => (min === null || size >= min) && (max === null || size <= max)
			);
			return {type: 'size', preset: preset?.preset ?? 'huge'};
		}
		case 'type':
			return file.kind === 'Directory'
				? {type: 'folders'}
				: {type: 'kind', kind: getContentKind(file)};
		case 'tags': {
			const tag = file.tags[0];
			return tag ? {type: 'tag', name: tag.canonical_name} : {type: 'untagged'};
		}
	}
}

/** Today, yesterday, the last 7 and 30 days, then months of this year, then years. */
export function dateLabel(value: string | null | undefined, now: Date): GroupLabel {
	const date = value ? new Date(value) : null;
	if (!date || Number.isNaN(date.getTime())) return {type: 'other'};

	const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
	const time = date.getTime();

	if (time >= startOfToday) return {type: 'date', bucket: 'today'};
	if (time >= startOfToday - DAY_MS) return {type: 'date', bucket: 'yesterday'};
	if (time >= startOfToday - 6 * DAY_MS) return {type: 'date', bucket: 'week'};
	if (time >= startOfToday - 29 * DAY_MS) return {type: 'date', bucket: 'month'};
	if (date.getFullYear() === now.getFullYear()) {
		return {type: 'month', year: date.getFullYear(), month: date.getMonth()};
	}
	return {type: 'year', year: date.getFullYear()};
}

function groupKey(label: GroupLabel): string {
	switch (label.type) {
		case 'letter':
			return `letter:${label.letter}`;
		case 'date':
			return `date:${label.bucket}`;
		case 'month':
			return `month:${label.year}-${label.month}`;
		case 'year':
			return `year:${label.year}`;
		case 'size':
			return `size:${label.preset}`;
		case 'kind':
			return `kind:${label.kind}`;
		case 'tag':
			return `tag:${label.name}`;
		default:
			return label.type;
	}
}

function fileName(file: File): string {
	return file.extension ? `${file.name}.${file.extension}` : file.name;
}

/** A virtualized grid row: a section header or up to `columns` cards. */
export type GridRow =
	| {type: 'header'; group: FileGroup}
	| {type: 'files'; start: number; files: File[]};

export interface GridLayout {
	rows: GridRow[];
	/** Row index of each file, by its position in the flat order. */
	rowOfFile: number[];
}

/**
 * Rows for the grid. Each section starts a new row so a header never shares one
 * with cards; without sections the files fill rows as before.
 */
export function buildGridLayout(
	groups: FileGroup[] | null,
	files: File[],
	columns: number
): GridLayout {
	const rows: GridRow[] = [];
	const rowOfFile: number[] = [];
	let start = 0;

	for (const group of groups ?? [{key: '', label: {type: 'other'}, files} as FileGroup]) {
		if (groups) rows.push({type: 'header', group});
		for (let offset = 0; offset < group.files.length; offset += columns) {
			const rowFiles = group.files.slice(offset, offset + columns);
			for (let i = 0; i < rowFiles.length; i++) rowOfFile.push(rows.length);
			rows.push({type: 'files', start, files: rowFiles});
			start += rowFiles.length;
		}
	}

	return {rows, rowOfFile};
}

/** The file above or below `index` in the same column, skipping headers. */
export function gridNeighbor(
	layout: GridLayout,
	index: number,
	direction: 'up' | 'down'
): number {
	const rowIndex = layout.rowOfFile[index];
	const row = rowIndex === undefined ? undefined : layout.rows[rowIndex];
	if (rowIndex === undefined || row?.type !== 'files') return index;

	const column = index - row.start;
	const step = direction === 'down' ? 1 : -1;
	for (let r = rowIndex + step; r >= 0 && r < layout.rows.length; r += step) {
		const candidate = layout.rows[r];
		if (candidate?.type === 'files') {
			return candidate.start + Math.min(column, candidate.files.length - 1);
		}
	}
	return index;
}

/** A virtualized list item: a section header or the file at `index` in the flat order. */
export type ListItem = {type: 'header'; group: FileGroup} | {type: 'file'; index: number};

export interface ListLayout {
	items: ListItem[];
	/** Item index of each file, by its position in the flat order. */
	itemOfFile: number[];
}

/**
 * Items for the list. `rowsPerGroup` counts each section's rows when expanded
 * folders add rows beneath their files; it defaults to the section's files.
 */
export function buildListLayout(
	groups: FileGroup[] | null,
	count: number,
	rowsPerGroup?: number[]
): ListLayout {
	const items: ListItem[] = [];
	const itemOfFile: number[] = [];

	if (!groups) {
		for (let index = 0; index < count; index++) {
			itemOfFile.push(index);
			items.push({type: 'file', index});
		}
		return {items, itemOfFile};
	}

	let index = 0;
	groups.forEach((group, groupIndex) => {
		items.push({type: 'header', group});
		const rows = rowsPerGroup?.[groupIndex] ?? group.files.length;
		for (let i = 0; i < rows; i++) {
			itemOfFile.push(items.length);
			items.push({type: 'file', index: index++});
		}
	});
	return {items, itemOfFile};
}
