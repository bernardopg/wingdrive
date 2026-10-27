import type {ColumnSizingState, SortingState} from '@tanstack/react-table';
import {
	getCoreRowModel,
	getSortedRowModel,
	useLegacyTable as useReactTable,
	type LegacyColumnDef as ColumnDef
} from '@tanstack/react-table/legacy';
import type {File} from '@wingdrive/ts-client';
import {useEffect, useMemo, useRef, useState} from 'react';
import {useTabManager} from '../../../../components/TabManager';
import {useExplorer} from '../../context';
import {formatBytes, formatRelativeTime} from '../../utils';
import {i18n} from '../../../../i18n';

export const ROW_HEIGHT = 36;
export const TABLE_PADDING_X = 16;
export const TABLE_PADDING_Y = 12;
export const TABLE_HEADER_HEIGHT = 32;

const columns: ColumnDef<File>[] = [
	{id: 'folders', accessorFn: (row) => (row.kind === 'Directory' ? 0 : 1)},
	{id: 'created', accessorFn: (row) => new Date(row.created_at).getTime()},
	{
		id: 'name',
		header: 'list.name',
		minSize: 200,
		size: 300,
		accessorFn: (row) =>
			row.name + (row.extension ? `.${row.extension}` : '')
	},
	{
		id: 'size',
		header: 'list.size',
		size: 80,
		minSize: 60,
		accessorFn: (row) => row.size,
		cell: (cell) =>
			cell.row.original.size > 0
				? formatBytes(cell.row.original.size)
				: '—'
	},
	{
		id: 'modified',
		header: 'list.modified',
		size: 120,
		minSize: 80,
		accessorFn: (row) => new Date(row.modified_at).getTime(),
		cell: (cell) => formatRelativeTime(cell.row.original.modified_at)
	},
	{
		id: 'type',
		header: 'list.kind',
		size: 80,
		minSize: 60,
		accessorFn: (row) =>
			row.kind === 'Directory'
				? i18n.t('explorer:list.folder')
				: row.kind === 'Symlink'
					? i18n.t('explorer:list.link')
					: row.extension?.toUpperCase() || i18n.t('explorer:list.file')
	},
	{
		id: 'tags',
		header: 'list.tags',
		size: 150,
		minSize: 80,
		accessorFn: (row) =>
			row.tags.map((tag) => tag.canonical_name).join(', ')
	}
];

const HIDDEN_COLUMNS = {folders: false, created: false};
const NAME_SIZE = 300;
const OTHER_COLUMNS = columns.filter((column) => column.header && column.id !== 'name');

/**
 * Until the user sizes the name column, it takes the width the other columns
 * leave, so the list spans the whole view at any window size.
 */
function fillNameColumn(
	sizing: ColumnSizingState,
	availableWidth: number
): ColumnSizingState {
	if (sizing.name !== undefined) return sizing;
	const others = OTHER_COLUMNS.reduce(
		(sum, column) => sum + (sizing[column.id!] ?? column.size!),
		0
	);
	return {
		...sizing,
		name: Math.max(NAME_SIZE, availableWidth - TABLE_PADDING_X * 2 - others)
	};
}

export function useTable(files: File[], availableWidth: number) {
	const {sortBy, sortDirection, viewSettings} = useExplorer();
	const {activeTabId, getExplorerState, updateExplorerState} =
		useTabManager();
	const listSorting = getExplorerState(activeTabId).listSorting;
	// The sorted rows are recomputed whenever this array changes identity, and
	// ListView publishes new rows to the Explorer, which renders it again.
	// A fresh array on every render therefore looped forever.
	const sorting = useMemo<SortingState>(
		() => [
			...(viewSettings.foldersFirst ? [{id: 'folders', desc: false}] : []),
			...(listSorting ?? [{id: sortBy, desc: sortDirection === 'Desc'}])
		],
		[viewSettings.foldersFirst, listSorting, sortBy, sortDirection]
	);
	const previousSort = useRef({sortBy, sortDirection});
	// The tab this list belongs to, even while another tab is active.
	const [tabId] = useState(activeTabId);
	const [columnSizing, setColumnSizing] = useState<ColumnSizingState>(
		() => getExplorerState(tabId).listColumnSizing ?? {}
	);
	useEffect(() => {
		if (
			previousSort.current.sortBy !== sortBy ||
			previousSort.current.sortDirection !== sortDirection
		) {
			previousSort.current = {sortBy, sortDirection};
			updateExplorerState(activeTabId, {
				listSorting: [{id: sortBy, desc: sortDirection === 'Desc'}]
			});
		}
	}, [sortBy, sortBy, sortDirection, activeTabId, updateExplorerState]);
	const filledSizing = useMemo(
		() => fillNameColumn(columnSizing, availableWidth),
		[columnSizing, availableWidth]
	);
	const core = useMemo(() => getCoreRowModel<File>(), []);
	const sorted = useMemo(() => getSortedRowModel<File>(), []);
	const table = useReactTable({
		data: files,
		columns,
		defaultColumn: {minSize: 60},
		getCoreRowModel: core,
		getSortedRowModel: sorted,
		columnResizeMode: 'onChange',
		enableSortingRemoval: false,
		state: {
			sorting,
			columnVisibility: HIDDEN_COLUMNS,
			columnSizing: filledSizing
		},
		onColumnSizingChange: (updater) => {
			setColumnSizing((previous) => {
				const filled = fillNameColumn(previous, availableWidth);
				const next =
					typeof updater === 'function' ? updater(filled) : updater;
				// Resizing another column leaves the name column filling the row.
				if (previous.name === undefined && next.name === filled.name) {
					const {name: _name, ...rest} = next;
					return rest;
				}
				return next;
			});
		},
		onSortingChange: (updater) => {
			const next: SortingState =
				typeof updater === 'function'
					? updater(table.getState().sorting)
					: updater;
			updateExplorerState(activeTabId, {
				listSorting: next.filter((entry) => entry.id !== 'folders')
			});
		},
		getRowId: (row) => row.id
	});
	// Save widths once a drag ends, not on every pointer move.
	const isResizing = !!table.getState().columnResizing.isResizingColumn;
	useEffect(() => {
		if (!isResizing)
			updateExplorerState(tabId, {listColumnSizing: columnSizing});
	}, [isResizing, columnSizing, tabId, updateExplorerState]);
	return {table, columns};
}
export type {ColumnSizingState};
