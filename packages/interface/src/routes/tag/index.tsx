import { useEffect } from 'react';
import { useParams } from 'react-router-dom';
import type { SearchTagsInput, SearchTagsOutput } from '@wingdrive/ts-client';
import { useTabManager } from '../../components/TabManager/useTabManager';
import { useNormalizedQuery } from '../../contexts/WingDriveContext';
import { useTranslation } from '../../i18n';
import { useExplorer } from '../explorer/context';
import { ExplorerView } from '../explorer/ExplorerView';

/**
 * Tag view — renders files tagged with a specific tag using the standard explorer view.
 * Activates tag mode on the explorer context so useExplorerFiles fetches from files.by_tag.
 */
export function TagView() {
	const { tagId } = useParams<{ tagId: string }>();
	const { enterTagMode, exitTagMode } = useExplorer();
	const { t } = useTranslation('dialogs');
	const { activeTabId, updateTabTitle } = useTabManager();
	const { data: tagName } = useNormalizedQuery<SearchTagsInput, SearchTagsOutput, string | null>({
		query: 'tags.search',
		input: { query: '' } as SearchTagsInput,
		resourceType: 'tag',
		select: (data) =>
			data?.tags
				?.map((result) => result.tag)
				.find((tag) => tag?.id === tagId)?.canonical_name ?? null
	});

	useEffect(() => {
		updateTabTitle(activeTabId, tagName ?? t('tabs.tag'));
	}, [tagName, activeTabId, updateTabTitle, t]);

	useEffect(() => {
		if (tagId) {
			enterTagMode(tagId);
		}
		return () => {
			exitTagMode();
		};
	}, [tagId, enterTagMode, exitTagMode]);

	return <ExplorerView />;
}
