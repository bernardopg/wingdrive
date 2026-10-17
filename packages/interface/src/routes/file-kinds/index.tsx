import {getIcon} from '@wingdrive/assets/util';
import {useIsDarkTheme} from '../../hooks/useTheme';
import type {ContentKindStatsOutput, SearchFilters} from '@wingdrive/ts-client';
import {useEffect, useMemo} from 'react';
import {useNavigate, useParams} from 'react-router-dom';
import {useNormalizedQuery} from '../../contexts/WingDriveContext';
import {ExplorerView, useExplorer} from '../explorer';
import {useTranslation} from '../../i18n';

// Map content kind names to icon names and colors
// Keys must match backend ContentKind variants (lowercase)
// Icon names must match actual files in packages/assets/icons/
const CONTENT_KIND_CONFIG: Record<string, {iconName: string; color: string}> = {
	image: {iconName: 'Image', color: '#3B82F6'},
	video: {iconName: 'Video', color: '#8B5CF6'},
	audio: {iconName: 'Audio', color: '#10B981'},
	document: {iconName: 'Document', color: '#F59E0B'},
	archive: {iconName: 'Archive', color: '#6366F1'},
	code: {iconName: 'Text', color: '#EF4444'}, // No Code.png, using Text.png
	text: {iconName: 'Text', color: '#6B7280'},
	database: {iconName: 'Database', color: '#14B8A6'},
	book: {iconName: 'Book', color: '#8B5CF6'},
	font: {iconName: 'Text', color: '#F59E0B'},
	mesh: {iconName: 'Mesh', color: '#06B6D4'},
	config: {iconName: 'Document', color: '#6366F1'},
	encrypted: {iconName: 'Encrypted', color: '#DC2626'},
	key: {iconName: 'Key', color: '#FCD34D'},
	executable: {iconName: 'Executable', color: '#7C3AED'},
	binary: {iconName: 'Executable', color: '#6B7280'},
	spreadsheet: {iconName: 'Document', color: '#10B981'},
	presentation: {iconName: 'Document', color: '#F97316'},
	email: {iconName: 'Document', color: '#3B82F6'},
	calendar: {iconName: 'Document', color: '#06B6D4'},
	contact: {iconName: 'Document', color: '#EC4899'},
	web: {iconName: 'Globe', color: '#3B82F6'},
	shortcut: {iconName: 'Link', color: '#8B5CF6'},
	package: {iconName: 'Package', color: '#F59E0B'},
	model_entry: {iconName: 'Mesh', color: '#06B6D4'},
	memory: {iconName: 'Database', color: '#6366F1'},
	unknown: {iconName: 'Document', color: '#6B7280'}
};

function formatFileCount(count: number): string {
	if (count >= 1000000) {
		return `${(count / 1000000).toFixed(1)}M`;
	}
	if (count >= 1000) {
		return `${(count / 1000).toFixed(1)}K`;
	}
	return count.toString();
}

/**
 * File Kinds View
 * Shows content kinds (images, videos, audio, etc.) with file counts
 */
export function FileKindsView() {
	const {t} = useTranslation('navigation');
	const isDark = useIsDarkTheme();
	const navigate = useNavigate();
	// Fetch content kind statistics
	const {data: statsData, isLoading} = useNormalizedQuery<
		Record<string, never>,
		ContentKindStatsOutput
	>({
		query: 'files.content_kind_stats',
		input: {},
		resourceType: 'content_kind'
	});

	const stats = (statsData?.stats ?? []).sort(
		(a, b) => Number(b.file_count) - Number(a.file_count)
	);
	const totalFiles = Number(statsData?.total_files ?? 0);
	const browsedFiles = stats.reduce((sum, stat) => sum + Number(stat.browsed_count), 0);

	if (isLoading) {
		return (
			<div className="flex h-full items-center justify-center">
				<span className="text-ink-dull">{t('fileKinds.loading')}</span>
			</div>
		);
	}

	return (
		<div className="flex h-full flex-col">
			{/* Header */}
			<div className="border-app-line border-b px-6 py-4">
				<div className="flex items-center justify-between">
					<div>
						<h1 className="text-ink text-2xl font-bold">
							{t('items.fileKinds')}
						</h1>
						<p className="text-ink-dull mt-1 text-sm">
							{t('fileKinds.description')}
						</p>
					</div>
					<div className="text-right">
						<div className="text-ink text-2xl font-bold">
							{formatFileCount(totalFiles)}
						</div>
						<div className="text-ink-dull text-xs">{t('fileKinds.totalFiles')}</div>
					</div>
				</div>
			</div>

			{/* Content Grid */}
			<div className="flex-1 overflow-auto p-3">
				{totalFiles === 0 && browsedFiles === 0 ? (
					<div className="flex h-full flex-col items-center justify-center text-center px-6">
						<p className="text-ink text-sm font-medium">
							{t('fileKinds.emptyTitle')}
						</p>
						<p className="text-ink-dull mt-1 max-w-sm text-xs">
							{t('fileKinds.emptyDescription')}
						</p>
					</div>
				) : (
				<div
					className="grid"
					style={{
						gridTemplateColumns:
							'repeat(auto-fill, minmax(140px, 1fr))',
						gap: '8px'
					}}
				>
					{stats.map((stat) => {
						const config =
							CONTENT_KIND_CONFIG[stat.name] ||
							CONTENT_KIND_CONFIG.unknown;
						const icon = getIcon(
							config.iconName,
							isDark,
							null,
							false
						);

						const fileCount = Number(stat.file_count);
						const browsedCount = Number(stat.browsed_count);

						return (
							<button
								key={stat.name}
								disabled={fileCount === 0}
								onClick={() =>
									navigate(`/file-kinds/${stat.name}`)
								}
								className="enabled:hover:bg-app-box/50 flex aspect-square flex-col items-center justify-center rounded-lg p-4 transition-colors disabled:cursor-default"
							>
								<img
									src={icon}
									alt={t(`fileKinds.types.${stat.name}`, {defaultValue: stat.name})}
									className="mb-3 h-16 w-16"
								/>
								<div className="w-full text-center">
										<div className="text-ink mb-1 text-sm font-medium capitalize">
											{t(`fileKinds.types.${stat.name}`, {defaultValue: stat.name})}
									</div>
									<div className="text-ink-dull text-xs">
										{formatFileCount(
											Number(stat.file_count)
										)}{' '}
											{t('fileKinds.fileCount', {count: fileCount})}
									</div>
									{browsedCount > 0 && (
										<div className="text-ink-faint mt-0.5 text-[11px]">
											{t('fileKinds.browsedCount', {count: formatFileCount(browsedCount)})}
										</div>
									)}
								</div>
							</button>
						);
					})}
				</div>
				)}
			</div>
		</div>
	);
}

const EMPTY_FILTERS: SearchFilters = {
	file_types: null,
	tags: null,
	date_range: null,
	size_range: null,
	locations: null,
	content_types: null,
	favorite: null,
	include_hidden: null,
	include_archived: null,
	at_risk: null,
	on_volumes: null,
	not_on_volumes: null,
	min_volume_count: null,
	max_volume_count: null
};

export function FileKindFiles() {
	const {t} = useTranslation('navigation');
	const {kindName} = useParams();
	const {enterFilteredMode, exitFilteredMode} = useExplorer();
	const {data, isLoading} = useNormalizedQuery<
		Record<string, never>,
		ContentKindStatsOutput
	>({
		query: 'files.content_kind_stats',
		input: {},
		resourceType: 'content_kind'
	});
	const stat = data?.stats.find(
		(item) => item.name === kindName && Number(item.file_count) > 0
	);
	const filters = useMemo<SearchFilters | null>(
		() => (stat ? {...EMPTY_FILTERS, content_types: [stat.kind]} : null),
		[stat]
	);

	useEffect(() => {
		if (!filters || !stat) return;
		enterFilteredMode(filters, t('fileKinds.kindFiles', {kind: t(`fileKinds.types.${stat.name}`, {defaultValue: stat.name})}));
		return exitFilteredMode;
	}, [enterFilteredMode, exitFilteredMode, filters, stat, t]);

	if (isLoading) {
		return (
			<div className="text-ink-dull flex h-full items-center justify-center">
				{t('fileKinds.loadingFiles')}
			</div>
		);
	}

	if (!stat) {
		return (
			<div className="text-ink-dull flex h-full items-center justify-center">
				{t('fileKinds.noFilesForKind')}
			</div>
		);
	}

	return <ExplorerView />;
}
