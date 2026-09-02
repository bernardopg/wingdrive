import {ArrowLeft, Plus} from '@phosphor-icons/react';
import {CircleButton, SearchBar} from '@wingdrive/primitives';
import {useMemo, useState} from 'react';
import {useNavigate} from 'react-router-dom';
import {SourceCard} from '../../components/Sources/SourceCard';
import {useTabManager} from '../../components/TabManager/useTabManager';
import {useLibraryQuery} from '../../contexts/SpacedriveContext';
import {TopBarItem, TopBarPortal} from '../../TopBar';

export function SourcesHome() {
	const navigate = useNavigate();
	const {createTab} = useTabManager();
	const [searchValue, setSearchValue] = useState('');
	const {
		data: sources,
		isLoading,
		error
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
						<TopBarItem id="back" label="Back" priority="high">
							<CircleButton
								icon={ArrowLeft}
								onClick={() => navigate(-1)}
							/>
						</TopBarItem>
						<TopBarItem id="title" label="Title" priority="high">
							<h1 className="text-ink text-xl font-bold">
								Sources
							</h1>
						</TopBarItem>
					</>
				}
				right={
					<>
						<TopBarItem id="search" label="Search" priority="high">
							<SearchBar
								placeholder="Search sources..."
								value={searchValue}
								onChange={setSearchValue}
								onClear={() => setSearchValue('')}
								className="w-64"
							/>
						</TopBarItem>
						<TopBarItem
							id="add-source"
							label="Add Source"
							priority="high"
						>
							<CircleButton
								icon={Plus}
								onClick={() =>
									createTab('Adapters', '/sources/adapters')
								}
								title="Add Source"
							/>
						</TopBarItem>
					</>
				}
			/>
			<div className="p-6">
				{isLoading && (
					<div className="flex items-center justify-center py-20">
						<div className="text-ink-faint text-sm">Loading...</div>
					</div>
				)}

				{error && (
					<div className="rounded-lg border border-red-400/20 p-4">
						<p className="text-sm text-red-400">
							Failed to load sources: {String(error)}
						</p>
					</div>
				)}

				{sources && sources.length === 0 && (
					<div className="flex flex-col items-center justify-center py-20">
						<p className="text-ink-dull text-sm">No sources yet</p>
						<p className="text-ink-faint mt-1 text-xs">
							Add a data source to get started
						</p>
						<button
							onClick={() =>
								createTab('Adapters', '/sources/adapters')
							}
							className="bg-accent hover:bg-accent-deep mt-4 rounded-lg px-3.5 py-1.5 text-sm font-medium text-white transition-colors"
						>
							Add Source
						</button>
					</div>
				)}

				{filteredSources &&
					filteredSources.length === 0 &&
					sources &&
					sources.length > 0 && (
						<div className="flex items-center justify-center py-20">
							<p className="text-ink-faint text-sm">
								No matching sources
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
