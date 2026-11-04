import type {Location} from '@wingdrive/ts-client';
import clsx from 'clsx';
import {AnimatePresence, motion} from 'framer-motion';
import {useEffect, useMemo} from 'react';
import {Outlet, useLocation, useParams} from 'react-router-dom';
import {Inspector} from './components/Inspector/Inspector';
import {JobsProvider} from './components/JobManager/hooks/JobsContext';
import {
	PREVIEW_LAYER_ID,
	QuickPreviewController,
	QuickPreviewSyncer
} from './components/QuickPreview';
import {SpacesSidebar} from './components/SpacesSidebar';
import {
	TabBar,
	LaunchRequestSync,
	TabDefaultsSync,
	TabNavigationSync
} from './components/TabManager';
import {usePane} from './components/TabManager/PaneContext';
import {usePlatform} from './contexts/PlatformContext';
import {useNormalizedQuery} from './contexts/WingDriveContext';
import {WebContextMenuProvider} from './contexts/WebContextMenuContext';
import {ExplorerProvider, useExplorer} from './routes/explorer';
import {KeyboardHandler} from './routes/explorer/KeyboardHandler';
import {FileContextMenuProvider} from './routes/explorer/FileContextMenuProvider';
import {OpenFileProvider} from './routes/explorer/OpenFileProvider';
import {SelectionProvider} from './routes/explorer/SelectionContext';
import {TagAssignmentMode} from './routes/explorer/TagAssignmentMode';
import {TopBar, TopBarProvider} from './TopBar';

/* Shared chrome metrics. The TopBar, floating tab bar and quick preview all
   offset themselves by these, and previously drifted 4px apart. */
const SIDEBAR_WIDTH = 220;
const INSPECTOR_WIDTH = 280;

function ShellLayoutContent() {
	const location = useLocation();
	const params = useParams();
	const platform = usePlatform();
	const pane = usePane();
	const {
		sidebarVisible: sidebarSetting,
		inspectorVisible: inspectorSetting,
		setInspectorVisible,
		quickPreviewFileId,
		tagModeActive,
		setTagModeActive,
		viewMode,
		currentPath
	} = useExplorer();

	// Check if we're on Overview (hide inspector) or in Knowledge view (has its own inspector)
	const isOverview = location.pathname === '/';
	const isKnowledgeView = viewMode === 'knowledge';

	// A split view shows the sidebar on the left pane and the Inspector on the
	// right one, so the window keeps one of each.
	const sidebarVisible = sidebarSetting && !(pane.split && pane.pane === 'secondary');
	const inspectorVisible = inspectorSetting && !(pane.split && pane.pane === 'primary');
	const showTabBar = !(pane.split && pane.pane === 'secondary');

	// Fetch locations to get current location info
	const locationsQuery = useNormalizedQuery<null, {locations: Location[]}>({
		query: 'locations.list',
		input: null,
		resourceType: 'location'
	});

	// Get current location if we're on a location route or browsing within a location
	const currentLocation = useMemo(() => {
		const locations = locationsQuery.data?.locations || [];

		// First try to match by route param (for /location/:id routes)
		if (params.locationId) {
			const loc = locations.find((loc) => loc.id === params.locationId);
			if (loc) return loc;
		}

		// If no route match, try to find location by matching current path
		if (currentPath && 'Physical' in currentPath) {
			const pathStr = currentPath.Physical.path;
			// Find location with longest matching prefix
			return (
				locations
					.filter((loc) => {
						if (!loc.wing_path || !('Physical' in loc.wing_path))
							return false;
						const locPath = loc.wing_path.Physical.path;
						return pathStr.startsWith(locPath);
					})
					.sort((a, b) => {
						const aPath =
							'Physical' in a.wing_path!
								? a.wing_path!.Physical.path
								: '';
						const bPath =
							'Physical' in b.wing_path!
								? b.wing_path!.Physical.path
								: '';
						return bPath.length - aPath.length;
					})[0] || null
			);
		}

		return null;
	}, [params.locationId, locationsQuery.data, currentPath]);

	useEffect(() => {
		// Listen for inspector window close events
		if (!platform.onWindowEvent) return;

		let unlisten: (() => void) | undefined;

		(async () => {
			try {
				unlisten = await platform.onWindowEvent!(
					'inspector-window-closed',
					() => {
						// Show embedded inspector when floating window closes
						setInspectorVisible(true);
					}
				);
			} catch (err) {
				console.error('Failed to setup inspector close listener:', err);
			}
		})();

		return () => {
			unlisten?.();
		};
	}, [platform, setInspectorVisible]);

	const handlePopOutInspector = async () => {
		if (!platform.showWindow) return;

		try {
			await platform.showWindow({
				type: 'Inspector',
				item_id: null
			});
			// Hide the embedded inspector when popped out
			setInspectorVisible(false);
		} catch (err) {
			console.error('Failed to pop out inspector:', err);
		}
	};

	const isPreviewActive = !!quickPreviewFileId;
	const isSizeViewActive = viewMode === 'size';

	return (
		<div
			className={clsx(
				'text-sidebar-ink bg-app relative flex h-screen select-none flex-col overflow-hidden border border-transparent',
				platform.platform === 'tauri' && 'rounded-[10px]',
			)}
		>
			{/* Preview layer - portal target for fullscreen preview, sits between content and sidebar/inspector */}
			<div
				id={PREVIEW_LAYER_ID}
				className="pointer-events-none absolute inset-0 z-40 [&>*]:pointer-events-auto"
			/>

			{/* Size view layer - portal target for fullscreen size view, sits below preview */}
			<div
				id="size-view-layer"
				className="pointer-events-none absolute inset-0 z-[35] [&>*]:pointer-events-auto"
			/>

			{/* Top fade mask - spans full width beyond sidebar/inspector */}
			<div
				className="pointer-events-none absolute left-0 right-0 top-0 z-[37] h-32 bg-gradient-to-b from-app to-transparent"
			/>

			<TopBar
				sidebarWidth={sidebarVisible ? SIDEBAR_WIDTH : 0}
				inspectorWidth={
					inspectorVisible && !isOverview && !isKnowledgeView
						? INSPECTOR_WIDTH
						: 0
				}
				isPreviewActive={isPreviewActive || isSizeViewActive}
			/>

			{/* Tab Bar floated above size view when active */}
			{isSizeViewActive && (
				<div
					className="pointer-events-none absolute left-0 right-0 z-[45] [&>*]:pointer-events-auto"
					style={{
						top: 48, // TopBar height
						paddingLeft: sidebarVisible ? SIDEBAR_WIDTH : 0,
						paddingRight:
							inspectorVisible && !isOverview && !isKnowledgeView
								? INSPECTOR_WIDTH
								: 0,
						transition: 'padding 0.3s ease-out'
					}}
				>
					{showTabBar && <TabBar />}
				</div>
			)}

			{/* Main content area with sidebar and content */}
			<div className="flex flex-1 overflow-hidden">
				<AnimatePresence initial={false} mode="popLayout">
					{sidebarVisible && (
						<motion.div
							initial={{x: -SIDEBAR_WIDTH, width: 0}}
							animate={{x: 0, width: SIDEBAR_WIDTH}}
							exit={{x: -SIDEBAR_WIDTH, width: 0}}
							transition={{
								duration: 0.3,
								ease: [0.25, 1, 0.5, 1]
							}}
							className="relative z-[65] overflow-hidden"
							// Sidebar clicks act on the focused pane and must not refocus.
							data-pane-neutral
						>
							<SpacesSidebar
								isPreviewActive={
									isPreviewActive || isSizeViewActive
								}
							/>
						</motion.div>
					)}
				</AnimatePresence>

				{/* Content area with tabs - positioned between sidebar and inspector */}
				<div
					className={clsx(
						'relative flex flex-1 flex-col overflow-hidden pt-12',
						isSizeViewActive ? 'z-[30]' : 'z-[38]'
					)}
				>
					{/* Tab Bar - nested inside content area like Finder (hidden in size view) */}
					{!isSizeViewActive && showTabBar && <TabBar />}

					{/* Router content renders here */}
					<div
						className={clsx(
							'relative flex-1 overflow-hidden',
							pane.split && 'rounded-lg ring-1 ring-inset',
							pane.split && (pane.focused ? 'ring-accent/60' : 'ring-transparent')
						)}
						data-pane={pane.pane}
						data-pane-focused={pane.focused}
					>
						<Outlet />

						{/* Tag Assignment Mode - positioned at bottom of main content area */}
						<TagAssignmentMode
							isActive={tagModeActive}
							onExit={() => setTagModeActive(false)}
						/>
					</div>
				</div>

				{/* Keyboard handler (invisible, doesn't cause parent rerenders) */}
				<KeyboardHandler />

				{/* Syncs selection to QuickPreview - isolated to prevent frame rerenders */}
				<QuickPreviewSyncer />

				<AnimatePresence initial={false}>
					{/* Hide inspector on Overview screen and Knowledge view (has its own) */}
					{inspectorVisible && !isOverview && !isKnowledgeView && (
						<motion.div
							initial={{width: 0}}
							animate={{width: INSPECTOR_WIDTH}}
							exit={{width: 0}}
							transition={{
								duration: 0.3,
								ease: [0.25, 1, 0.5, 1]
							}}
							className="relative z-[65] overflow-hidden"
						>
							<div className="flex h-full w-[280px] min-w-[280px] flex-col bg-transparent p-2">
								{/* w-[280px] must match INSPECTOR_WIDTH */}
								<Inspector
									currentLocation={currentLocation}
									onPopOut={handlePopOutInspector}
									isPreviewActive={
										isPreviewActive || isSizeViewActive
									}
								/>
							</div>
						</motion.div>
					)}
				</AnimatePresence>
			</div>

			{/* Quick Preview - isolated component to prevent frame rerenders on selection change */}
			<QuickPreviewController
				sidebarWidth={sidebarVisible ? SIDEBAR_WIDTH : 0}
				inspectorWidth={
					inspectorVisible && !isOverview && !isKnowledgeView
						? INSPECTOR_WIDTH
						: 0
				}
			/>
		</div>
	);
}

/** Window-wide work, such as opening folders requested by the launcher, runs in one pane. */
function PrimaryPaneOnly({children}: {children: React.ReactNode}) {
	return usePane().pane === 'primary' ? children : null;
}

export function ShellLayout() {
	return (
		<JobsProvider>
			<TopBarProvider>
				<SelectionProvider>
					<ExplorerProvider>
						<WebContextMenuProvider>
							<FileContextMenuProvider>
								<OpenFileProvider>
									{/* Sync tab navigation and defaults with router */}
									<TabNavigationSync />
									<TabDefaultsSync />
									<PrimaryPaneOnly>
										<LaunchRequestSync />
									</PrimaryPaneOnly>
									<ShellLayoutContent />
								</OpenFileProvider>
							</FileContextMenuProvider>
						</WebContextMenuProvider>
					</ExplorerProvider>
				</SelectionProvider>
			</TopBarProvider>
		</JobsProvider>
	);
}
