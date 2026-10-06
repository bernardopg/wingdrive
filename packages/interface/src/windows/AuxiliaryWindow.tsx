import {Dialogs, Toaster, TooltipProvider} from '@wingdrive/primitives';
import type {ReactNode} from 'react';
import {ErrorBoundary} from '../components/ErrorBoundary';
import {JobsProvider} from '../components/JobManager/hooks/JobsContext';
import {DaemonDisconnectedOverlay} from '../components/overlays/DaemonDisconnectedOverlay';
import {DaemonStartupOverlay} from '../components/overlays/DaemonStartupOverlay';
import {usePlatform} from '../contexts/PlatformContext';
import {ServerProvider} from '../contexts/ServerContext';
import {
	WingDriveProvider,
	type WingDriveClient
} from '../contexts/WingDriveContext';
import {useDaemonStatus} from '../hooks/useDaemonStatus';
import {useLiveFileEvents} from '../hooks/useLiveFileEvents';
import {useTheme} from '../hooks/useTheme';

function WindowEffects() {
	useTheme();
	useLiveFileEvents();
	return null;
}

function DaemonGate({children}: {children: ReactNode}) {
	const daemonStatus = useDaemonStatus();
	if (daemonStatus.isConnected) return <>{children}</>;
	return daemonStatus.isStarting ? (
		<DaemonStartupOverlay show />
	) : (
		<DaemonDisconnectedOverlay daemonStatus={daemonStatus} />
	);
}

/**
 * Shared shell for secondary windows (Settings, Jobs, Inspector, Quick
 * Preview, Spacebot).
 *
 * These windows run their own React tree, so without this shell they missed
 * the main window's toasts, dialogs, jobs context, theme and live file
 * events: toasts never showed, confirmation dialogs never opened and the Jobs
 * window crashed outside JobsProvider. Content waits for the daemon like the
 * main window does and shows the same reconnect controls when it drops.
 */
export function AuxiliaryWindow({
	client,
	children
}: {
	client: WingDriveClient;
	children: ReactNode;
}) {
	const platform = usePlatform();

	return (
		<WingDriveProvider client={client}>
			<ServerProvider>
				<TooltipProvider>
					<WindowEffects />
					<ErrorBoundary>
						{platform.platform === 'tauri' ? (
							<DaemonGate>
								<JobsProvider>{children}</JobsProvider>
							</DaemonGate>
						) : (
							<JobsProvider>{children}</JobsProvider>
						)}
					</ErrorBoundary>
					<Dialogs />
					<Toaster />
				</TooltipProvider>
			</ServerProvider>
		</WingDriveProvider>
	);
}
