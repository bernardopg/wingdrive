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
	const explorer = useExplorer();
	const { viewMode, mode, setSearchPage } = explorer;
	const { files, isLoading, error, hasMore, page = 0 } = useExplorerFiles();

	// Only render if we're in search mode
	if (mode.type !== 'search') {
		return null;
	}

	// Show minimum character hint
	if (mode.query.length < MIN_SEARCH_LENGTH) {
		return (
			<div className="flex h-full flex-col items-center justify-center p-8 text-center">
				<p className="text-ink-dull text-sm">
					Type at least 2 characters to search
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
			<div key={page} className="relative min-h-0 flex-1">
				{view}
			</div>
			{error && (
				<p role="alert" className="text-ink p-2 text-sm">
					Search failed: {error.message}
				</p>
			)}
			{!isLoading && !error && files.length === 0 && (
				<p className="text-ink-dull p-2 text-center text-sm">
					No results{page > 0 ? ' on this page' : ''}.
				</p>
			)}
			{(page > 0 || hasMore) && (
				<nav
					aria-label="Search result pages"
					className="text-ink flex items-center justify-center gap-3 p-2 text-sm"
				>
					<button
						type="button"
						disabled={page === 0 || isLoading}
						onClick={() => setSearchPage(page - 1)}
					>
						Previous
					</button>
					<span>Page {page + 1}</span>
					<button
						type="button"
						disabled={!hasMore || isLoading}
						onClick={() => setSearchPage(page + 1)}
					>
						Next
					</button>
				</nav>
			)}
		</div>
	);
}
