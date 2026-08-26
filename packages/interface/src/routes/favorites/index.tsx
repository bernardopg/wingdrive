import {Star} from '@phosphor-icons/react';
import type {SearchFilters} from '@sd/ts-client';
import {useEffect, type ReactNode} from 'react';
import {ExplorerView, useExplorer} from '../explorer';
import {useExplorerFiles} from '../explorer/hooks/useExplorerFiles';

const FAVORITE_FILTERS: SearchFilters = {
	file_types: null,
	tags: null,
	date_range: null,
	size_range: null,
	locations: null,
	content_types: null,
	favorite: true,
	include_hidden: null,
	include_archived: null,
	at_risk: null,
	on_volumes: null,
	not_on_volumes: null,
	min_volume_count: null,
	max_volume_count: null
};

export function FavoritesView() {
	const {mode, enterFilteredMode, exitFilteredMode} = useExplorer();

	useEffect(() => {
		enterFilteredMode(FAVORITE_FILTERS, 'Favorites');
		return exitFilteredMode;
	}, [enterFilteredMode, exitFilteredMode]);

	const {files, isLoading, error} = useExplorerFiles();
	const isActive = mode.type === 'filtered' && mode.label === 'Favorites';

	if (!isActive || isLoading) {
		return <StateMessage>Loading favorites...</StateMessage>;
	}

	if (error) {
		return (
			<StateMessage>
				Could not load favorites. {error.message}
			</StateMessage>
		);
	}

	if (files.length === 0) {
		return (
			<StateMessage>
				<Star size={36} weight="thin" />
				<span>No favorites yet.</span>
				<span className="text-xs">
					Mark a file as favorite from its inspector.
				</span>
			</StateMessage>
		);
	}

	return <ExplorerView />;
}

function StateMessage({children}: {children: ReactNode}) {
	return (
		<div className="text-ink-dull flex h-full flex-col items-center justify-center gap-2">
			{children}
		</div>
	);
}
