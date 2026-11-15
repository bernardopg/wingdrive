import {useDndContext, useDroppable} from '@dnd-kit/core';
import {
	SortableContext,
	useSortable,
	verticalListSortingStrategy
} from '@dnd-kit/sortable';
import {CSS} from '@dnd-kit/utilities';
import {
	ArrowsClockwise,
	ArrowsOut,
	CircleNotch,
	FunnelSimple,
	GearSix,
	ListBullets,
	Palette,
	Trash,
} from '@phosphor-icons/react';
import {useSidebarStore} from '@wingdrive/ts-client';
import type {
	Wing,
	WingGroup as WingGroupType,
	WingItem as WingItemType
} from '@wingdrive/ts-client';
import {CircleButton, Popover, usePopover} from '@wingdrive/primitives';
import clsx from 'clsx';
import {motion} from 'framer-motion';
import {memo, useEffect, useState} from 'react';
import {useSidebarNavigate} from '../TabManager/useSidebarNavigate';
import {useTranslation} from '../../i18n';
import {usePlatform} from '../../contexts/PlatformContext';
import {useWingDriveClient} from '../../contexts/WingDriveContext';
import {useLibraries} from '../../hooks/useLibraries';
import {JobList} from '../JobManager/components/JobList';
import {useJobsContext} from '../JobManager/hooks/JobsContext';
import {CARD_HEIGHT} from '../JobManager/types';
import {ActivityFeed} from '../SyncMonitor/components/ActivityFeed';
import {PeerList} from '../SyncMonitor/components/PeerList';
import {useSyncCount} from '../SyncMonitor/hooks/useSyncCount';
import {useSyncMonitor} from '../SyncMonitor/hooks/useSyncMonitor';
import {AddGroupButton} from './AddGroupButton';
import {useWingLayout, useWings} from './hooks/useWings';
import {WingCustomizationPanel} from './WingCustomizationPanel';
import {FolderTree} from './FolderTree/FolderTree';
import {sidebarSections} from './reorderItems';
import {WingGroup} from './WingGroup';
import {WingItem} from './WingItem';
import {WingSwitcher} from './WingSwitcher';

// Wrapper that adds a wing-level drop zone before each group and makes it sortable
function WingGroupWithDropZone({
	group,
	items,
	wingId,
	isFirst: _isFirst
}: {
	group: WingGroupType;
	items: WingItemType[];
	wingId?: string;
	isFirst: boolean;
}) {
	const {active} = useDndContext();

	// Disable drop zone when dragging groups or wing items (they have 'label' in their data)
	// This allows sortable collision detection to work for reordering
	const isDraggingSortableItem = active?.data?.current?.label != null;

	const {setNodeRef: setDropRef, isOver} = useDroppable({
		id: `wing-root-before-${group.id}`,
		disabled: !wingId || isDraggingSortableItem,
		data: {
			action: 'add-to-wing',
			wingId,
			groupId: null
		}
	});

	// Sortable for group reordering
	const {
		attributes,
		listeners,
		setNodeRef: setSortableRef,
		transform,
		transition,
		isDragging,
		setActivatorNodeRef: _setActivatorNodeRef
	} = useSortable({
		id: group.id,
		data: {
			label: group.name
		}
	});

	const style = {
		transform: CSS.Transform.toString(transform),
		transition
	};

	return (
		<div
			ref={setSortableRef}
			style={style}
			className={clsx('relative', isDragging && 'z-50 opacity-50')}
		>
			{/* Drop zone before this group (for adding root-level items) */}
			<div
				ref={setDropRef}
				className="absolute -top-2.5 left-0 right-0 z-10 h-5"
			>
				{isOver && !isDragging && !isDraggingSortableItem && (
					<div className="bg-accent absolute left-2 right-2 top-1/2 h-[2px] -translate-y-1/2 rounded-full" />
				)}
			</div>
			<WingGroup
				group={group}
				items={items}
				wingId={wingId}
				sortableAttributes={attributes}
				sortableListeners={listeners}
			/>
		</div>
	);
}

function SortableFolderTree({id}: {id: string}) {
	const {t} = useTranslation('navigation');
	const {attributes, listeners, setNodeRef, transform, transition, isDragging} = useSortable({
		id, data: {label: t('folderTree.title')}
	});
	return (
		<div ref={setNodeRef} style={{transform: CSS.Transform.toString(transform), transition}} className={clsx(isDragging && 'relative z-50 opacity-50')}>
			<FolderTree sortableAttributes={attributes} sortableListeners={listeners} />
		</div>
	);
}

// Sync Monitor Button with Popover
const SyncButton = memo(function SyncButton() {
const {t} = useTranslation('navigation');
	const popover = usePopover();
	const navigate = useSidebarNavigate();
	const [showActivityFeed, setShowActivityFeed] = useState(false);
	const {onlinePeerCount, isSyncing} = useSyncCount();
	const sync = useSyncMonitor();

	useEffect(() => {
		if (popover.open) {
			setShowActivityFeed(false);
		}
	}, [popover.open]);

	const getStateColor = (state: string) => {
		switch (state) {
			case 'Ready':
				return 'bg-green-500';
			case 'Backfilling':
				return 'bg-yellow-500';
			case 'CatchingUp':
				return 'bg-accent';
			case 'Uninitialized':
				return 'bg-ink-faint';
			case 'Paused':
				return 'bg-ink-dull';
			default:
				return 'bg-ink-faint';
		}
	};

	return (
		<Popover.Root open={popover.open} onOpenChange={popover.setOpen}>
			<Popover.Trigger asChild>
				<CircleButton
					icon={({className, ...props}) =>
						isSyncing ? (
							<CircleNotch
								className={clsx(className, 'animate-spin')}
								{...props}
							/>
						) : (
							<ArrowsClockwise className={className} {...props} />
						)
					}
					title={t('sync.title')}
				/>
			</Popover.Trigger>
			<Popover.Content
				side="top"
				align="start"
				sideOffset={8}
				className="!bg-app z-50 max-h-[520px] w-[380px] !rounded-xl !p-0"
			>
				<div className="border-app-line flex items-center justify-between border-b px-4 py-3">
					<h3 className="text-ink text-sm font-semibold">
						{t('sync.title')}
					</h3>

					<div className="flex items-center gap-2">
						{onlinePeerCount > 0 && (
							<span className="text-ink-dull text-xs">
								{t('sync.onlinePeers', {count: onlinePeerCount})}
							</span>
						)}

						<CircleButton
							icon={ArrowsOut}
							onClick={() => navigate('/sync')}
							title={t('sync.openFull')}
						/>

						<CircleButton
							icon={FunnelSimple}
							active={showActivityFeed}
							onClick={() =>
								setShowActivityFeed(!showActivityFeed)
							}
							title={
								showActivityFeed
									? t('sync.showPeers')
									: t('sync.showActivity')
							}
						/>
					</div>
				</div>

				{popover.open && (
					<>
						<div className="border-app-line bg-app-box/50 border-b px-4 py-2">
							<div className="flex items-center gap-2">
								<div
									className={`size-2 rounded-full ${getStateColor(sync.currentState)}`}
								/>
								<span className="text-ink-dull text-xs font-medium">
									{t(`sync.states.${sync.currentState}`)}
								</span>
							</div>
						</div>
						<motion.div
							className="no-scrollbar overflow-y-auto"
							initial={false}
							animate={{
								height: showActivityFeed
									? Math.min(
											sync.recentActivity.length * 40 +
												16,
											400
										)
									: Math.min(sync.peers.length * 80 + 16, 400)
							}}
							transition={{
								duration: 0.2,
								ease: [0.25, 1, 0.5, 1]
							}}
						>
							{showActivityFeed ? (
								<ActivityFeed
									activities={sync.recentActivity}
								/>
							) : (
								<PeerList
									peers={sync.peers}
									currentState={sync.currentState}
								/>
							)}
						</motion.div>
					</>
				)}
			</Popover.Content>
		</Popover.Root>
	);
});

// Jobs Button with Popover
const JobsButton = memo(
	function JobsButton({
		activeJobCount,
		hasRunningJobs,
		jobs,
		pause,
		resume,
		cancel,
		getSpeedHistory,
		navigate
	}: {
		activeJobCount: number;
		hasRunningJobs: boolean;
		jobs: any[];
		pause: (jobId: string) => Promise<void>;
		resume: (jobId: string) => Promise<void>;
		cancel: (jobId: string) => Promise<void>;
		getSpeedHistory: (jobId: string) => any[];
		navigate: any;
	}) {
	const {t} = useTranslation('navigation');
		const popover = usePopover();
		const [showOnlyRunning, setShowOnlyRunning] = useState(true);

		useEffect(() => {
			if (popover.open) {
				setShowOnlyRunning(true);
			}
		}, [popover.open]);

		const filteredJobs = showOnlyRunning
			? jobs.filter(
					(job) => job.status === 'running' || job.status === 'paused'
				)
			: jobs;

		return (
			<Popover.Root open={popover.open} onOpenChange={popover.setOpen}>
				<Popover.Trigger asChild>
					<CircleButton
						icon={({className, ...props}) =>
							hasRunningJobs ? (
								<CircleNotch
									className={clsx(className, 'animate-spin')}
									{...props}
								/>
							) : (
								<ListBullets className={className} {...props} />
							)
						}
						title={t('jobs.title')}
					/>
				</Popover.Trigger>
				<Popover.Content
					side="top"
					align="start"
					sideOffset={8}
					className="!bg-app z-50 max-h-[480px] w-[360px] !rounded-xl !p-0"
				>
					<div className="border-app-line flex items-center justify-between border-b px-4 py-3">
						<h3 className="text-ink text-sm font-semibold">
							{t('jobs.title')}
						</h3>

						<div className="flex items-center gap-2">
							{activeJobCount > 0 && (
								<span className="text-ink-dull text-xs">
									{t('jobs.activeCount', {count: activeJobCount})}
								</span>
							)}

							<CircleButton
								icon={ArrowsOut}
								onClick={() => navigate('/jobs')}
								title={t('jobs.openFull')}
							/>

							<CircleButton
								icon={FunnelSimple}
								active={showOnlyRunning}
								onClick={() =>
									setShowOnlyRunning(!showOnlyRunning)
								}
								title={
									showOnlyRunning
										? t('jobs.showAll')
										: t('jobs.showActive')
								}
							/>
						</div>
					</div>

					{popover.open && (
						<motion.div
							className="no-scrollbar overflow-y-auto"
							initial={false}
							animate={{
								height:
									filteredJobs.length === 0
										? CARD_HEIGHT + 16
										: Math.min(
												filteredJobs.length *
													(CARD_HEIGHT + 8) +
													16,
												400
											)
							}}
							transition={{
								duration: 0.2,
								ease: [0.25, 1, 0.5, 1]
							}}
						>
							<JobList
								jobs={filteredJobs}
								onPause={pause}
								onResume={resume}
								onCancel={cancel}
								getSpeedHistory={getSpeedHistory}
							/>
						</motion.div>
					)}
				</Popover.Content>
			</Popover.Root>
		);
	},
	(prevProps, nextProps) =>
		// The job list carries progress; ignoring it froze the open popover on
		// the first snapshot. The callbacks are recreated every render, so
		// comparing them would defeat the memo.
		prevProps.activeJobCount === nextProps.activeJobCount &&
		prevProps.hasRunningJobs === nextProps.hasRunningJobs &&
		prevProps.jobs === nextProps.jobs
);

interface WingsSidebarProps {
	isPreviewActive?: boolean;
}

// Memoized: the shell re-renders on every selection and navigation change,
// and the sidebar only depends on its own queries and contexts.
export const WingsSidebar = memo(function WingsSidebar({
	isPreviewActive = false
}: WingsSidebarProps) {
	const {t} = useTranslation('navigation');
	const client = useWingDriveClient();
	const platform = usePlatform();
	const navigate = useSidebarNavigate();
	const {data: libraries} = useLibraries();
	const [currentLibraryId, setCurrentLibraryId] = useState<string | null>(
		() => client.getCurrentLibraryId()
	);
	const [customizePanelOpen, setCustomizePanelOpen] = useState(false);

	// Get sync and job status for icons
	useSyncCount();
	const {
		activeJobCount,
		hasRunningJobs,
		jobs,
		pause,
		resume,
		cancel,
		getSpeedHistory
	} = useJobsContext();

	const {currentWingId, setCurrentWing, folderTreePositions} = useSidebarStore();
	const {data: wingsData} = useWings();
	const wings = (wingsData as any)?.wings as Wing[] | undefined;

	// Listen for library changes from client and update local state
	useEffect(() => {
		const handleLibraryChange = (newLibraryId: string) => {
			setCurrentLibraryId(newLibraryId);
		};

		client.on('library-changed', handleLibraryChange);
		return () => {
			client.off('library-changed', handleLibraryChange);
		};
	}, [client]);

	// Auto-select the first library when none (or an unknown one) is selected
	useEffect(() => {
		// Also covers a remembered library that no longer exists
		const known = libraries?.some((lib) => lib.id === currentLibraryId);
		if (libraries && libraries.length > 0 && !known) {
			const firstLib = libraries[0];

			// Set library ID via platform (syncs to all windows on Tauri)
			if (platform.setCurrentLibraryId) {
				platform
					.setCurrentLibraryId(firstLib.id)
					.catch((err) =>
						console.error('Failed to set library ID:', err)
					);
			} else {
				// Web fallback - just update client
				client.setCurrentLibrary(firstLib.id);
			}
		}
	}, [libraries, currentLibraryId, client, platform]);

	// Auto-select first wing if none selected
	const currentWing =
		wings?.find((s) => s.id === currentWingId) ?? wings?.[0];

	useEffect(() => {
		if (currentWing && currentWing.id !== currentWingId) {
			setCurrentWing(currentWing.id);
		}
	}, [currentWing, currentWingId, setCurrentWing]);

	const {data: layoutData} = useWingLayout(currentWing?.id ?? null);
	const layout = layoutData as { wing_items: WingItemType[]; groups: Array<{ group: WingGroupType; items: WingItemType[] }> } | undefined;

	return (
		<div className="flex h-full w-[220px] min-w-[176px] max-w-[300px] flex-col bg-transparent p-2">
			<div
				className={clsx(
					'flex h-full flex-col overflow-hidden rounded-2xl',
					isPreviewActive
						? 'bg-sidebar/80 backdrop-blur-2xl'
						: 'bg-sidebar/65'
				)}
			>
				<nav
					className={clsx(
						"relative z-[51] flex h-full flex-col gap-2.5 p-2.5 pb-2",
						platform.platform === "tauri" && "pt-[43px]",
					)}
				>
					{/* Wing Switcher */}
					<WingSwitcher
						wings={wings}
						currentWing={currentWing}
						onSwitch={setCurrentWing}
						onOpenSettings={() => setCustomizePanelOpen(true)}
					/>

					{/* Scrollable Content */}
					<div className="no-scrollbar mask-fade-out mt-3 flex grow flex-col space-y-5 overflow-x-hidden overflow-y-scroll pb-10">
						{/* Wing-level items (pinned shortcuts) */}
						{layout?.wing_items &&
							layout.wing_items.length > 0 && (
								<SortableContext
									items={layout.wing_items.map(
										(item) => item.id
									)}
									strategy={verticalListSortingStrategy}
								>
									<div className="space-y-0.5">
										{layout.wing_items.map(
											(item, index) => (
												<WingItem
													key={item.id}
													item={item}
													isLastItem={
														index ===
														layout.wing_items
															.length -
															1
													}
													allowInsertion={true}
													wingId={currentWing?.id}
													groupId={null}
													sortable={true}
												/>
											)
										)}
									</div>
								</SortableContext>
							)}

						{layout?.groups && currentWing && (() => {
							const treeId = `folder-tree:${currentWing.id}`;
							const sections = sidebarSections(layout.groups.map(({group}) => group.id), treeId, folderTreePositions[currentWing.id]);
							return (
								<SortableContext items={sections} strategy={verticalListSortingStrategy}>
									{sections.map((id, index) => {
										if (id === treeId) return <SortableFolderTree key={id} id={id} />;
										const entry = layout.groups.find(({group}) => group.id === id)!;
										return <WingGroupWithDropZone key={id} group={entry.group} items={entry.items} wingId={currentWing.id} isFirst={index === 0} />;
									})}
								</SortableContext>
							);
						})()}

						{/* Add Group Button */}
						{currentWing && (
							<AddGroupButton wingId={currentWing.id} />
						)}
					</div>

					{/* Sync Monitor, Job Manager, Customize & Settings (pinned to bottom) */}
					<div className="grid grid-cols-5 items-center justify-items-center">
						<SyncButton />
							<JobsButton
								activeJobCount={activeJobCount}
								hasRunningJobs={hasRunningJobs}
								jobs={jobs}
								pause={pause}
								resume={resume}
								cancel={cancel}
								getSpeedHistory={getSpeedHistory}
								navigate={navigate}
							/>
							<CircleButton
								icon={Palette}
								title={t('sidebar.customize')}
								onClick={() => setCustomizePanelOpen(true)}
							/>
						<CircleButton
							icon={Trash}
							title={t('sidebar.trash')}
							onClick={() => navigate('/trash')}
						/>
						<CircleButton
							icon={GearSix}
							title={t('sidebar.settings')}
							onClick={() => {
								if (platform.showWindow) {
									platform
										.showWindow({
											type: 'Settings',
											page: 'general'
										})
										.catch((err) =>
											console.error(
												'Failed to open settings:',
												err
											)
										);
								} else {
									window.open('/settings/general', '_blank');
								}
							}}
						/>
					</div>
				</nav>
			</div>

			{/* Customization Panel */}
			<WingCustomizationPanel
				isOpen={customizePanelOpen}
				onClose={() => setCustomizePanelOpen(false)}
				wingId={currentWing?.id ?? null}
			/>
		</div>
	);
});
