import type {File} from '@wingdrive/ts-client';
import {useMemo} from 'react';
import {useExplorer} from '../context';
import {filterFiles} from './folderFilter';

/**
 * Applies the folder filter to a view's listing while browsing. Every view
 * filters its own list, since media, size and column views do not show the
 * plain folder listing. `unfilteredCount` is set only while the filter narrows.
 */
export function useFolderFilter(files: File[]): {files: File[]; unfilteredCount?: number} {
	const {folderFilter, mode} = useExplorer();
	const text = mode.type === 'browse' ? folderFilter?.trim() : '';
	return useMemo(
		() =>
			text ? {files: filterFiles(files, text), unfilteredCount: files.length} : {files},
		[files, text]
	);
}
