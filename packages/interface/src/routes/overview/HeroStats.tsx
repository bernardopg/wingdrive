import {CaretLeft, CaretRight} from '@phosphor-icons/react';
import DatabaseIcon from '@wingdrive/assets/icons/Database.png';
import DevicesIcon from '@wingdrive/assets/icons/Devices.png';
import IndexedIcon from '@wingdrive/assets/icons/Indexed.png';
import LocationIcon from '@wingdrive/assets/icons/Location.png';
import MobileIcon from '@wingdrive/assets/icons/Mobile.png';
import StorageIcon from '@wingdrive/assets/icons/Storage.png';
import TagsIcon from '@wingdrive/assets/icons/Tags.png';
import {CircleButton} from '@wingdrive/primitives';
import {motion} from 'framer-motion';
import {useEffect, useRef, useState} from 'react';
import {useTranslation} from '../../i18n';
import { ThemedIcon } from '../../components/ThemedIcon';

interface HeroStatsProps {
	totalStorage: number; // bytes
	usedStorage: number; // bytes
	totalFiles: number;
	locationCount: number;
	tagCount: number;
	deviceCount: number;
	uniqueContentCount: number;
	databaseSize: number; // bytes
	sidecarCount: number;
	sidecarSize: number; // bytes
}

function formatBytes(bytes: number): {value: string; unit: string} {
	if (bytes === 0) return {value: '0', unit: 'B'};
	const k = 1024;
	const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
	const i = Math.floor(Math.log(bytes) / Math.log(k));
	return {
		value: (bytes / Math.pow(k, i)).toFixed(1),
		unit: sizes[i]
	};
}

export function HeroStats({
	totalStorage,
	usedStorage,
	totalFiles,
	locationCount,
	tagCount,
	deviceCount,
	uniqueContentCount,
	databaseSize,
	sidecarCount,
	sidecarSize
}: HeroStatsProps) {
	const {t} = useTranslation('navigation');
	const scrollRef = useRef<HTMLDivElement>(null);
	const [canScrollLeft, setCanScrollLeft] = useState(false);
	const [canScrollRight, setCanScrollRight] = useState(false);

	const usagePercent =
		totalStorage > 0 ? (usedStorage / totalStorage) * 100 : 0;

	const storageFormatted = formatBytes(totalStorage);
	const usedFormatted = formatBytes(usedStorage);
	const databaseFormatted = formatBytes(databaseSize);
	const sidecarFormatted = formatBytes(sidecarSize);

	const updateScrollState = () => {
		if (!scrollRef.current) return;
		const {scrollLeft, scrollWidth, clientWidth} = scrollRef.current;
		setCanScrollLeft(scrollLeft > 0);
		setCanScrollRight(scrollLeft < scrollWidth - clientWidth - 1);
	};

	useEffect(() => {
		updateScrollState();
		window.addEventListener('resize', updateScrollState);
		return () => window.removeEventListener('resize', updateScrollState);
	}, []);

	const scroll = (direction: 'left' | 'right') => {
		if (!scrollRef.current) return;
		// Scroll 4 cards at a time (280px per card + 32px gap)
		const cardWidth = 280;
		const gap = 32;
		const cardsPerPage = 4;
		const scrollAmount = (cardWidth + gap) * cardsPerPage;
		scrollRef.current.scrollBy({
			left: direction === 'left' ? -scrollAmount : scrollAmount,
			behavior: 'smooth'
		});
	};

	return (
		<div className="px-8 py-8">
			<div className="relative">
				{/* Left fade and button */}
				{canScrollLeft && (
					<>
						<div className="pointer-events-none absolute bottom-0 left-0 top-0 z-10 w-24 bg-gradient-to-r from-app to-transparent" />
						<div className="absolute left-2 top-1/2 z-20 -translate-y-1/2">
							<CircleButton
								icon={CaretLeft}
								onClick={() => scroll('left')}
							/>
						</div>
					</>
				)}

				{/* Scrollable container */}
				<div
					ref={scrollRef}
					onScroll={updateScrollState}
					className="scrollbar-hide flex gap-8 overflow-x-auto"
					style={{scrollbarWidth: 'none'}}
				>
				{/* Total Storage */}
				<StatCard
					icon={
						<ThemedIcon
							src={DevicesIcon}
							alt={t('overview.storage')}
							className="size-10 opacity-80"
							// style={{ filter: 'drop-shadow(0 0 4px rgba(217, 70, 239, 0.4))' }}
						/>
					}
					label={t('overview.totalStorage')}
					value={
						<>
							{storageFormatted.value}{' '}
							<span className="text-ink-faint text-xl">
								{storageFormatted.unit}
							</span>
						</>
					}
					subtitle={
						<>
							<span className="text-accent">
								{usedFormatted.value}{' '}
								<span className="text-accent/70 text-[10px]">
									{usedFormatted.unit}
								</span>
							</span>{' '}
							{t('overview.used')}
						</>
					}
					progress={usagePercent}
					color="from-accent to-cyan-500"
				/>

				{/* Files */}
				<StatCard
					icon={
						<ThemedIcon
							src={IndexedIcon}
							alt={t('overview.files')}
							className="size-10 opacity-80"
						/>
					}
					label={t('overview.filesIndexed')}
					value={totalFiles.toLocaleString()}
					subtitle={t('overview.uniqueFiles', {count: uniqueContentCount, formattedCount: uniqueContentCount.toLocaleString()})}
					color="from-purple-500 to-pink-500"
				/>

				{/* Devices */}
				<StatCard
					icon={
						<ThemedIcon
							src={MobileIcon}
							alt={t('sidebar.devices')}
							className="size-10 opacity-80"
						/>
					}
					label={t('overview.connectedDevices')}
					value={deviceCount}
					subtitle={t('overview.registeredInLibrary')}
					color="from-green-500 to-emerald-500"
				/>

				{/* Library Size (Database) */}
				<StatCard
					icon={
						<ThemedIcon
							src={DatabaseIcon}
							alt={t('sidebar.library')}
							className="size-10 opacity-80"
						/>
					}
					label={t('overview.librarySize')}
					value={
						<>
							{databaseFormatted.value}{' '}
							<span className="text-ink-faint text-xl">
								{databaseFormatted.unit}
							</span>
						</>
					}
					subtitle={t('overview.databaseOnDisk')}
					color="from-blue-500 to-cyan-500"
				/>

				{/* Sidecar Storage */}
				<StatCard
					icon={
						<ThemedIcon
							src={StorageIcon}
							alt={t('overview.sidecars')}
							className="size-10 opacity-80"
						/>
					}
					label={t('overview.sidecarStorage')}
					value={
						<>
							{sidecarFormatted.value}{' '}
							<span className="text-ink-faint text-xl">
								{sidecarFormatted.unit}
							</span>
						</>
					}
					subtitle={t('overview.filesGenerated', {count: sidecarCount, formattedCount: sidecarCount.toLocaleString()})}
					color="from-orange-500 to-red-500"
				/>

				{/* Locations */}
				<StatCard
					icon={
						<ThemedIcon
							src={LocationIcon}
							alt={t('sidebar.locations')}
							className="size-10 opacity-80"
						/>
					}
					label={t('sidebar.locations')}
					value={locationCount}
					subtitle={t('overview.indexedFolders')}
					color="from-teal-500 to-green-500"
				/>

				{/* Tags */}
				<StatCard
					icon={
						<ThemedIcon
							src={TagsIcon}
							alt={t('sidebar.tags')}
							className="size-10 opacity-80"
						/>
					}
					label={t('sidebar.tags')}
					value={tagCount}
					subtitle={t('overview.organizationLabels')}
					color="from-pink-500 to-rose-500"
				/>
			</div>

			{/* Right fade and button */}
			{canScrollRight && (
				<>
					<div className="pointer-events-none absolute bottom-0 right-0 top-0 z-10 w-24 bg-gradient-to-l from-app to-transparent" />
					<div className="absolute right-2 top-1/2 z-20 -translate-y-1/2">
						<CircleButton
							icon={CaretRight}
							onClick={() => scroll('right')}
						/>
					</div>
				</>
			)}
		</div>
		</div>
	);
}

interface StatCardProps {
	icon: React.ReactNode;
	label: string;
	value: string | number | React.ReactNode;
	subtitle: React.ReactNode;
	progress?: number;
	pulse?: boolean;
	color: string;
	badge?: string;
}

function StatCard({
	icon,
	label,
	value,
	subtitle,
	progress: _progress,
	pulse,
	color: _color,
	badge
}: StatCardProps) {
	return (
		<div className="relative min-w-[280px] flex-shrink-0">
			{badge && (
				<div className="bg-sidebar-box text-sidebar-ink border-sidebar-line absolute -right-2 -top-2 rounded-full border px-2 py-0.5 text-xs font-medium">
					{badge}
				</div>
			)}

			<div className="flex gap-3">
				<div className="relative">
					{icon}
					{pulse && (
						<motion.div
							animate={{
								scale: [1, 1.2, 1],
								opacity: [1, 0.5, 1]
							}}
							transition={{duration: 2, repeat: Infinity}}
							className="bg-accent absolute -right-1 -top-1 size-2 rounded-full"
						/>
					)}
				</div>
				<div className="flex-1">
					<div className="text-ink mb-1 text-3xl font-bold">
						{value}
					</div>
					<div className="text-ink-dull mb-1 text-xs">{label}</div>
					<div className="text-ink-faint text-xs">{subtitle}</div>
				</div>
			</div>
		</div>
	);
}
