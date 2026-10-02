import type {ColumnSizingState, SortingState} from '@tanstack/react-table';
import {
	getCoreRowModel,
	getSortedRowModel,
	useLegacyTable as useReactTable,
	type LegacyColumnDef as ColumnDef
} from '@tanstack/react-table/legacy';
import type {File} from '@wingdrive/ts-client';
import {useEffect, useMemo, useRef} from 'react';
import {useTabManager} from '../../../../components/TabManager';
import {useExplorer} from '../../context';
import {formatBytes, formatRelativeTime} from '../../utils';

export const ROW_HEIGHT = 36;
export const TABLE_PADDING_X = 16;
export const TABLE_PADDING_Y = 12;
export const TABLE_HEADER_HEIGHT = 32;

const columns: ColumnDef<File>[] = [
	{id: 'folders', accessorFn: (row) => (row.kind === 'Directory' ? 0 : 1)},
	{id: 'created', accessorFn: (row) => new Date(row.created_at).getTime()},
	{
		id: 'name',
		header: 'Name',
		minSize: 200,
		size: 300,
		maxSize: 800,
		accessorFn: (row) =>
			row.name + (row.extension ? `.${row.extension}` : '')
	},
	{
		id: 'size',
		header: 'Size',
		size: 80,
		minSize: 60,
		maxSize: 120,
		accessorFn: (row) => row.size,
		cell: (cell) =>
			cell.row.original.size > 0
				? formatBytes(cell.row.original.size)
				: '—'
	},
	{
		id: 'modified',
		header: 'Modified',
		size: 120,
		minSize: 80,
		maxSize: 180,
		accessorFn: (row) => new Date(row.modified_at).getTime(),
		cell: (cell) => formatRelativeTime(cell.row.original.modified_at)
	},
	{
		id: 'type',
		header: 'Kind',
		size: 80,
		minSize: 60,
		maxSize: 120,
		accessorFn: (row) =>
			row.kind === 'Directory'
				? 'Folder'
				: row.kind === 'Symlink'
					? 'Link'
					: row.extension?.toUpperCase() || 'File'
	},
	{
		id: 'tags',
		header: 'Tags',
		size: 150,
		minSize: 80,
		maxSize: 400,
		accessorFn: (row) =>
			row.tags.map((tag) => tag.canonical_name).join(', ')
	}
];

export function useTable(files: File[]) {
	const {sortBy, sortDirection, viewSettings} = useExplorer();
	const {activeTabId, getExplorerState, updateExplorerState} =
		useTabManager();
	const defaultSorting = [{id: sortBy, desc: sortDirection === 'Desc'}];
	const savedSorting =
		getExplorerState(activeTabId).listSorting ?? defaultSorting;
	const previousSort = useRef({sortBy, sortDirection});
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
	const core = useMemo(() => getCoreRowModel<File>(), []);
	const sorted = useMemo(() => getSortedRowModel<File>(), []);
	const table = useReactTable({
		data: files,
		columns,
		defaultColumn: {minSize: 60, maxSize: 500},
		getCoreRowModel: core,
		getSortedRowModel: sorted,
		columnResizeMode: 'onChange',
		enableSortingRemoval: false,
		state: {
			sorting: [
				...(viewSettings.foldersFirst
					? [{id: 'folders', desc: false}]
					: []),
				...savedSorting
			],
			columnVisibility: {folders: false, created: false}
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
	return {table, columns};
}
export type {ColumnSizingState};
