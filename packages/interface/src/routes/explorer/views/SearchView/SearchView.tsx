import type { ReactNode } from 'react';
import { useExplorer } from '../../context';
import { useExplorerFiles } from '../../hooks/useExplorerFiles';
import { MIN_SEARCH_LENGTH } from '../../hooks/searchQuery';
import { GridView } from '../GridView';
import { ListView } from '../ListView';
import { MediaView } from '../MediaView';
import { ColumnView } from '../ColumnView';
import { SizeView } from '../SizeView';
import { KnowledgeView } from '../KnowledgeView';
import { useTranslation } from '../../../../i18n';
import { useLibraryQuery } from '../../../../contexts/WingDriveContext';
import { SourceSearchResults } from '../../../../components/Sources/SourceSearchResults';
import { hasSearchableWord } from '../../../../components/Sources/sourceSearch';

/** Source hits one library search asks for, across every source. */
const SOURCE_HIT_LIMIT = 60;

/**
 * SearchView is a router that delegates to the appropriate view component.
 *
 * Instead of reimplementing rendering logic, it reuses existing view components
 * (GridView, ListView, etc.) which now automatically handle search results via
 * the useExplorerFiles hook.
 *
 * This ensures search has the same interactions as normal browsing:
 * - Keyboard navigation (arrow keys, shift-select)
 * - Drag-to-select
 * - Context menus
 * - Quick preview
 * - All other explorer features
 */
export function SearchView() {
	const { t } = useTranslation('explorer');
	const explorer = useExplorer();
	const { viewMode, mode, setSearchPage } = explorer;
	const { files, isLoading, error, hasMore, page = 0 } = useExplorerFiles();
	// Only a library search reaches sources: folder and location scopes are places on disk.
	const sourceQuery =
		mode.type === 'search' && mode.scope === 'library' ? mode.query.trim() : '';
	const sourceSearch = useLibraryQuery(
		{
			type: 'sources.search',
			input: { query: sourceQuery, source_id: null, limit: SOURCE_HIT_LIMIT }
		},
		{
			enabled:
				sourceQuery.length >= MIN_SEARCH_LENGTH && hasSearchableWord(sourceQuery),
			placeholderData: (previous) => previous
		}
	);
	const sourceHits = sourceQuery ? (sourceSearch.data?.hits ?? []) : [];

	// Only render if we're in search mode
	if (mode.type !== 'search') {
		return null;
	}

	// Show minimum character hint
	if (mode.query.length < MIN_SEARCH_LENGTH) {
		return (
			<div className="flex h-full flex-col items-center justify-center p-8 text-center">
				<p className="text-ink-dull text-sm">
					{t('search.minimumCharacters')}
				</p>
			</div>
		);
	}

	let view: ReactNode;
	switch (viewMode) {
		case 'list':
			view = <ListView />;
			break;
		case 'media':
			view = <MediaView />;
			break;
		case 'column':
			view = <ColumnView />;
			break;
		case 'size':
			view = <SizeView />;
			break;
		case 'knowledge':
			view = <KnowledgeView />;
			break;
		default:
			view = <GridView />;
	}

	return (
		<div className="flex h-full flex-col">
			{page === 0 && <SourceSearchResults query={sourceQuery} hits={sourceHits} />}
			<div key={page} className="relative min-h-0 flex-1">
				{view}
			</div>
			{error && (
				<p role="alert" className="text-ink p-2 text-sm">
					{t('errors.searchFailed', { error: error.message })}
				</p>
			)}
			{!isLoading && !error && files.length === 0 && (
				<p className="text-ink-dull p-2 text-center text-sm">
					{t(
						page > 0
							? 'search.noResultsThisPage'
							: sourceHits.length > 0
								? 'search.noFileResults'
								: 'search.noResults'
					)}
				</p>
			)}
			{(page > 0 || hasMore) && (
				<nav
					aria-label={t('search.resultPages')}
					className="text-ink flex items-center justify-center gap-3 p-2 text-sm"
				>
					<button
						type="button"
						disabled={page === 0 || isLoading}
						onClick={() => setSearchPage(page - 1)}
					>
						{t('search.previous')}
					</button>
					<span>{t('search.page', { count: page + 1 })}</span>
					<button
						type="button"
						disabled={!hasMore || isLoading}
						onClick={() => setSearchPage(page + 1)}
					>
						{t('search.next')}
					</button>
				</nav>
			)}
		</div>
	);
}
