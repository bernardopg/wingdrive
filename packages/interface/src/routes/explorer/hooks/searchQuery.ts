import type {
	ContentKind,
	FileSearchInput,
	SortDirection,
	SortField,
	WingPath
} from '@wingdrive/ts-client';
import type {SearchScope, SortBy} from '../context';

/** Results per search page. One extra row is requested to detect a next page. */
export const SEARCH_PAGE_SIZE = 200;

/** Delay before a typed query runs, so each keystroke does not start a search. */
export const SEARCH_DEBOUNCE_MS = 300;

export const MIN_SEARCH_LENGTH = 2;

/** Content kinds offered as search filters, in display order. */
export const SEARCH_CONTENT_KINDS: {kind: ContentKind; label: string}[] = [
	{kind: 'image', label: 'Images'},
	{kind: 'video', label: 'Videos'},
	{kind: 'audio', label: 'Audio'},
	{kind: 'document', label: 'Documents'},
	{kind: 'text', label: 'Text'},
	{kind: 'code', label: 'Code'},
	{kind: 'archive', label: 'Archives'}
];

export function searchSortField(sortBy: SortBy): SortField {
	switch (sortBy) {
		case 'name':
			return 'Name';
		case 'size':
			return 'Size';
		case 'modified':
			return 'ModifiedAt';
		case 'created':
			return 'CreatedAt';
		default:
			return 'Relevance';
	}
}

interface LocationLike {
	wing_path: WingPath;
}

function physical(path: WingPath | null | undefined) {
	return path && 'Physical' in path ? path.Physical : null;
}

/**
 * Root of the location that contains `path`, or null when the path is outside
 * every location. The deepest root wins for nested locations, and a root only
 * matches whole path segments, so /data/photos2 is not inside /data/photos.
 */
export function findLocationRoot(
	path: WingPath | null,
	locations: readonly LocationLike[]
): WingPath | null {
	const target = physical(path);
	if (!target) return null;

	let best: WingPath | null = null;
	let bestLength = -1;
	for (const location of locations) {
		const root = physical(location.wing_path);
		if (!root || root.device_slug !== target.device_slug) continue;
		const rootPath = root.path.replace(/\/+$/, '');
		const inside =
			target.path === rootPath ||
			target.path.startsWith(`${rootPath}/`) ||
			rootPath === '';
		if (inside && rootPath.length > bestLength) {
			best = location.wing_path;
			bestLength = rootPath.length;
		}
	}
	return best;
}

export interface SearchRequest {
	query: string;
	scope: SearchScope;
	currentPath: WingPath | null;
	locationRoot: WingPath | null;
	sortBy: SortBy;
	sortDirection: SortDirection;
	includeHidden: boolean;
	contentTypes: ContentKind[];
	page: number;
}

/**
 * Builds the daemon search input for the toolbar state. Folder and location
 * scopes fall back to the whole library only when there is no folder to
 * search under, and the scope bar says so.
 */
export function buildSearchInput(request: SearchRequest): FileSearchInput {
	const scopePath =
		request.scope === 'folder'
			? request.currentPath
			: request.scope === 'location'
				? request.locationRoot
				: null;

	return {
		query: request.query,
		scope: scopePath ? {Path: {path: scopePath}} : 'Library',
		filters: {
			file_types: null,
			tags: null,
			date_range: null,
			size_range: null,
			locations: null,
			content_types:
				request.contentTypes.length > 0 ? request.contentTypes : null,
			favorite: null,
			include_hidden: request.includeHidden,
			include_archived: null,
			at_risk: null,
			on_volumes: null,
			not_on_volumes: null,
			min_volume_count: null,
			max_volume_count: null
		},
		mode: 'Normal',
		sort: {
			field: searchSortField(request.sortBy),
			direction: request.sortDirection
		},
		pagination: {
			limit: SEARCH_PAGE_SIZE + 1,
			offset: request.page * SEARCH_PAGE_SIZE
		}
	};
}
