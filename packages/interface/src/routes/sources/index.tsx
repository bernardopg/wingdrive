import {ArrowLeft, Plus} from '@phosphor-icons/react';
import {CircleButton, SearchBar} from '@wingdrive/primitives';
import {useMemo, useState} from 'react';
import {useNavigate} from 'react-router-dom';
import {SourceCard} from '../../components/Sources/SourceCard';
import {useTabManager} from '../../components/TabManager/useTabManager';
import {useLibraryQuery} from '../../contexts/WingDriveContext';
import {TopBarItem, TopBarPortal} from '../../TopBar';
import {useTranslation} from '../../i18n';

export function SourcesHome() {
	const {t} = useTranslation('navigation');
	const navigate = useNavigate();
	const {createTab} = useTabManager();
	const [searchValue, setSearchValue] = useState('');
	const {
		data: sources,
		isLoading,
		error,
		refetch,
		isFetching
	} = useLibraryQuery({
		type: 'sources.list',
		input: {data_type: null}
	});
	const filteredSources = useMemo(() => {
		const query = searchValue.trim().toLowerCase();
		if (!sources || !query) return sources;

		return sources.filter((source) =>
			[source.name, source.adapter_id, source.data_type]
				.filter(Boolean)
				.some((value) => value.toLowerCase().includes(query))
		);
	}, [searchValue, sources]);

	return (
		<>
			<TopBarPortal
				left={
					<>
						<TopBarItem id="back" label={t('common.back')} priority="high">
							<CircleButton
								icon={ArrowLeft}
								onClick={() => navigate(-1)}
							/>
						</TopBarItem>
						<TopBarItem id="title" label={t('common.title')} priority="high">
							<h1 className="text-ink text-xl font-bold">
								{t('items.sources')}
							</h1>
						</TopBarItem>
					</>
				}
				right={
					<>
						<TopBarItem id="search" label={t('common.search')} priority="high">
							<SearchBar
								placeholder={t('sources.searchPlaceholder')}
								value={searchValue}
								onChange={setSearchValue}
								onClear={() => setSearchValue('')}
								className="w-64"
							/>
						</TopBarItem>
						<TopBarItem
							id="add-source"
							label={t('sources.add')}
							priority="high"
						>
							<CircleButton
								icon={Plus}
								onClick={() =>
									createTab('Adapters', '/sources/adapters')
								}
								title={t('sources.add')}
							/>
						</TopBarItem>
					</>
				}
			/>
			<div className="p-6">
				{isLoading && (
					<div className="flex items-center justify-center py-20">
						<div className="text-ink-faint text-sm">{t('common.loading')}</div>
					</div>
				)}

				{error && (
					<div className="rounded-lg border border-red-400/20 p-4">
						<p className="text-sm text-red-400">
							{t('sources.loadFailed', {error: String(error)})}
						</p>
						<button
							type="button"
							disabled={isFetching}
							onClick={() => void refetch()}
							className="text-ink mt-2 text-xs underline disabled:opacity-50"
						>
							{isFetching ? t('common.retrying') : t('common.retry')}
						</button>
					</div>
				)}

				{sources && sources.length === 0 && (
					<div className="flex flex-col items-center justify-center py-20">
						<p className="text-ink-dull text-sm">{t('sources.empty')}</p>
						<p className="text-ink-faint mt-1 text-xs">
							{t('sources.emptyHint')}
						</p>
						<button
							onClick={() =>
								createTab('Adapters', '/sources/adapters')
							}
							className="bg-accent hover:bg-accent-deep mt-4 rounded-lg px-3.5 py-1.5 text-sm font-medium text-white transition-colors"
						>
							{t('sources.add')}
						</button>
					</div>
				)}

				{filteredSources &&
					filteredSources.length === 0 &&
					sources &&
					sources.length > 0 && (
						<div className="flex items-center justify-center py-20">
							<p className="text-ink-faint text-sm">
								{t('sources.noMatches')}
							</p>
						</div>
					)}

				{filteredSources && filteredSources.length > 0 && (
					<div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
						{filteredSources.map((source) => (
							<SourceCard key={source.id} source={source} />
						))}
					</div>
				)}
			</div>
		</>
	);
}
