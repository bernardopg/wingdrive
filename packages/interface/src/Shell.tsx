import {ReactQueryDevtools} from '@tanstack/react-query-devtools';
import {Dialogs, Toaster, TooltipProvider} from '@wingdrive/primitives';
import {DndProvider} from './components/DndProvider';
import {DaemonDisconnectedOverlay} from './components/overlays/DaemonDisconnectedOverlay';
import {DaemonStartupOverlay} from './components/overlays/DaemonStartupOverlay';
import {
	TabKeyboardHandler,
	TabManagerProvider,
	TabView
} from './components/TabManager';
import {usePlatform} from './contexts/PlatformContext';
import {ServerProvider} from './contexts/ServerContext';
import {
	WingDriveProvider,
	type WingDriveClient
} from './contexts/WingDriveContext';
import {useDaemonStatus} from './hooks/useDaemonStatus';
import {useLiveFileEvents} from './hooks/useLiveFileEvents';
import {useTheme} from './hooks/useTheme';
import {explorerRoutes} from './router';

interface ShellProps {
	client: WingDriveClient;
}

function ThemeApplier() {
	useTheme();
	return null;
}

function ShellWithTabs() {
	return (
		<DndProvider>
			<ThemeApplier />
			<TabView />
		</DndProvider>
	);
}

/**
 * Tauri-specific wrapper that prevents Shell from rendering until daemon is connected.
 * This avoids the connection storm where hundreds of queries try to execute before daemon is ready.
 */
function ShellWithDaemonCheck() {
	const daemonStatus = useDaemonStatus();
	const {isConnected, isStarting} = daemonStatus;

	return (
		<>
			{isConnected ? (
				// Daemon connected - render full app
				<>
					<TabManagerProvider routes={explorerRoutes}>
						<TabKeyboardHandler />
						<ShellWithTabs />
					</TabManagerProvider>
					<Dialogs />
					<Toaster />
					<ReactQueryDevtools
						initialIsOpen={false}
						buttonPosition="bottom-right"
					/>
				</>
			) : (
				// Daemon not connected - show appropriate overlay
				<>
					<DaemonStartupOverlay show={isStarting} />
					{!isStarting && (
						<DaemonDisconnectedOverlay
							daemonStatus={daemonStatus}
						/>
					)}
				</>
			)}
		</>
	);
}

function LiveFileEventsInvalidator() {
	// Must be inside WingDriveProvider + QueryClientProvider.
	useLiveFileEvents();
	return null;
}

export function Shell({client}: ShellProps) {
	const platform = usePlatform();
	const isTauri = platform.platform === 'tauri';

	return (
		<WingDriveProvider client={client}>
			<LiveFileEventsInvalidator />
			<ServerProvider>
				<TooltipProvider>
					{isTauri ? (
						// Tauri: Wait for daemon connection before rendering content
						<ShellWithDaemonCheck />
					) : (
						// Web: Render immediately (daemon connection handled differently)
						<>
							<TabManagerProvider routes={explorerRoutes}>
								<TabKeyboardHandler />
								<ShellWithTabs />
							</TabManagerProvider>
							<Dialogs />
							<Toaster />
							<ReactQueryDevtools
								initialIsOpen={false}
								buttonPosition="bottom-right"
							/>
						</>
					)}
				</TooltipProvider>
			</ServerProvider>
		</WingDriveProvider>
	);
}
