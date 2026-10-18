import { useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useExplorer } from '../../context';
import { useExplorerFiles } from '../../hooks/useExplorerFiles';
import { ExplorerView } from '../../ExplorerView';
import { useTranslation } from '../../../../i18n';

/**
 * RecentsView displays recently indexed files sorted by indexed_at timestamp.
 *
 * It renders the regular ExplorerView in recents mode, so recents keeps the
 * explorer's top bar (navigation, view switcher, sort) and its interactions.
 */
export function RecentsView() {
	const { t } = useTranslation('explorer');
	const explorer = useExplorer();
	const { enterRecentsMode, exitRecentsMode } = explorer;
	const { files, isLoading, error, hasMore, page = 0, source } = useExplorerFiles();
	const [, setSearchParams] = useSearchParams();

	// Enter recents mode on mount, exit on unmount
	useEffect(() => {
		enterRecentsMode();
		return () => exitRecentsMode();
	}, [enterRecentsMode, exitRecentsMode]);

	const goToPage = (nextPage: number) => setSearchParams(params => {
		params.set('page', String(nextPage));
		return params;
	});
	return <div className="flex h-full flex-col">
		<div key={page} className="min-h-0 flex-1"><ExplorerView /></div>
		{error && <p role="alert" className="p-2 text-sm">{t('pagination.couldNotLoadRecentFiles', { error: error.message })}</p>}
		{source === 'recents' && !isLoading && !error && files.length === 0 && <p className="p-2 text-sm">{t(page > 0 ? 'pagination.noRecentFilesThisPage' : 'pagination.noRecentFiles')}</p>}
		{source === 'recents' && (page > 0 || hasMore) && <nav aria-label={t('pagination.recentFilesPages')} className="flex items-center justify-center gap-3 p-2 text-sm">
			<button type="button" disabled={page === 0 || isLoading} onClick={() => goToPage(page - 1)}>{t('pagination.previous')}</button>
			<span>{t('pagination.page', { count: page + 1 })}</span>
			<button type="button" disabled={!hasMore || isLoading} onClick={() => goToPage(page + 1)}>{t('pagination.next')}</button>
		</nav>}
	</div>;
}
