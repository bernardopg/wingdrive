import { useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useExplorer } from '../../context';
import { useExplorerFiles } from '../../hooks/useExplorerFiles';
import { GridView } from '../GridView';
import { ListView } from '../ListView';
import { MediaView } from '../MediaView';
import { ColumnView } from '../ColumnView';
import { SizeView } from '../SizeView';
import { KnowledgeView } from '../KnowledgeView';

/**
 * RecentsView displays recently indexed files sorted by indexed_at timestamp.
 *
 * Similar to SearchView, it delegates to existing view components which automatically
 * read from useExplorerFiles. This ensures recents has the same interactions as normal
 * browsing: keyboard navigation, drag-to-select, context menus, etc.
 */
export function RecentsView() {
	const explorer = useExplorer();
	const { viewMode, enterRecentsMode, exitRecentsMode } = explorer;
	const { files, isLoading, error, hasMore, page = 0, source } = useExplorerFiles();
	const [, setSearchParams] = useSearchParams();

	// Enter recents mode on mount, exit on unmount
	useEffect(() => {
		enterRecentsMode();
		return () => exitRecentsMode();
	}, [enterRecentsMode, exitRecentsMode]);

	// Route to the appropriate view based on viewMode
	// The views will automatically use recents results via useExplorerFiles
	let view;
	switch (viewMode) {
		case 'grid':
			view = <GridView />;
			break;
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
	const goToPage = (nextPage: number) => setSearchParams(params => {
		params.set('page', String(nextPage));
		return params;
	});
	return <div className="flex h-full flex-col">
		<div key={page} className="min-h-0 flex-1">{view}</div>
		{error && <p role="alert" className="p-2 text-sm">Could not load recent files: {error.message}</p>}
		{source === 'recents' && !isLoading && !error && files.length === 0 && <p className="p-2 text-sm">No recent files{page > 0 ? ' on this page' : ''}.</p>}
		{source === 'recents' && (page > 0 || hasMore) && <nav aria-label="Recent files pages" className="flex items-center justify-center gap-3 p-2 text-sm">
			<button type="button" disabled={page === 0 || isLoading} onClick={() => goToPage(page - 1)}>Previous</button>
			<span>Page {page + 1}</span>
			<button type="button" disabled={!hasMore || isLoading} onClick={() => goToPage(page + 1)}>Next</button>
		</nav>}
	</div>;
}
