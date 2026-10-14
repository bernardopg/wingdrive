import { useEffect, useState } from 'react';
import { Power, Check, Warning, CircleNotch } from '@phosphor-icons/react';
import { usePlatform } from '../../contexts/PlatformContext';
import { useTranslation } from '../../i18n';

interface DaemonStatus {
	is_running: boolean;
	socket_path: string;
	server_url: string | null;
	started_by_us: boolean;
}

export function DaemonManager() {
const { t } = useTranslation('navigation');
	const platform = usePlatform();
	const [status, setStatus] = useState<DaemonStatus | null>(null);
	const [isLoading, setIsLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [autoStart, setAutoStart] = useState(true);
	const [runInProcess, setRunInProcess] = useState(false);
	const [isStarting, setIsStarting] = useState(false);
	const [isStopping, setIsStopping] = useState(false);

	useEffect(() => {
		checkDaemonStatus();
	}, []);

	async function checkDaemonStatus() {
		if (!platform.getDaemonStatus) return;

		setIsLoading(true);
		setError(null);
		try {
			const daemonStatus = await platform.getDaemonStatus();
			setStatus(daemonStatus);
		} catch (err) {
			setError(err instanceof Error ? err.message : String(err));
			setStatus(null);
		} finally {
			setIsLoading(false);
		}
	}

	async function handleStartDaemon() {
		if (!platform.startDaemonProcess) return;

		setIsStarting(true);
		setError(null);
		try {
			await platform.startDaemonProcess();
			await checkDaemonStatus();
		} catch (err) {
			setError(err instanceof Error ? err.message : String(err));
		} finally {
			setIsStarting(false);
		}
	}

	async function handleStopDaemon() {
		if (!platform.stopDaemonProcess) return;

		setIsStopping(true);
		setError(null);
		try {
			await platform.stopDaemonProcess();
			await checkDaemonStatus();
		} catch (err) {
			setError(err instanceof Error ? err.message : String(err));
		} finally {
			setIsStopping(false);
		}
	}

	async function handleOpenSettings() {
		if (!platform.openMacOSSettings) return;

		try {
			await platform.openMacOSSettings();
		} catch (err) {
			setError(err instanceof Error ? err.message : String(err));
		}
	}

	function getStatusColor() {
		if (isLoading) return 'text-ink-faint';
		return status?.is_running ? 'text-green-500' : 'text-red-500';
	}

	function getStatusIcon() {
		if (isLoading) return CircleNotch;
		return status?.is_running ? Check : Warning;
	}

	const StatusIcon = getStatusIcon();
	const isRunning = status?.is_running || false;

	return (
		<div className="flex flex-col h-full p-6 gap-6 text-ink">
			{/* Header */}
			<div className="flex items-center justify-between">
				<div>
				<h1 className="text-2xl font-semibold text-ink">{t('daemon.title')}</h1>
					<p className="text-sm text-ink-dull mt-1">
						{t('daemon.description')}
					</p>
				</div>
			</div>

			{/* Status Card */}
			<div className="bg-app-box border border-app-line rounded-lg p-4">
				<div className="flex items-center justify-between mb-4">
				<h2 className="text-lg font-medium text-ink">{t('daemon.status')}</h2>
					<div className="flex items-center gap-2">
						<StatusIcon
							className={`size-5 ${getStatusColor()} ${
								isLoading ? 'animate-spin' : ''
							}`}
							weight="fill"
						/>
						<span className={`text-sm font-medium ${getStatusColor()}`}>
							{isLoading ? t('daemon.checking') : isRunning ? t('daemon.running') : t('daemon.stopped')}
						</span>
					</div>
				</div>

				{error && (
					<div className="bg-red-500/10 border border-red-500/20 rounded-md p-3 mb-4">
						<p className="text-sm text-red-400">{error}</p>
					</div>
				)}

				<div className="space-y-2 text-sm">
					<div className="flex justify-between">
						<span className="text-ink-dull">{t('daemon.socketPath')}</span>
						<span className="text-ink font-mono text-xs">
							{status?.socket_path || t('common.notAvailable')}
						</span>
					</div>
					<div className="flex justify-between">
						<span className="text-ink-dull">{t('daemon.serverUrl')}</span>
						<span className="text-ink font-mono text-xs">
							{status?.server_url || t('common.notAvailable')}
						</span>
					</div>
					<div className="flex justify-between">
						<span className="text-ink-dull">{t('daemon.startedByApp')}</span>
						<span className="text-ink">
							{status?.started_by_us ? t('common.yes') : t('common.no')}
						</span>
					</div>
				</div>

				<button
					onClick={checkDaemonStatus}
					className="mt-4 w-full px-4 py-2 bg-accent hover:bg-accent-deep text-white rounded-md text-sm font-medium transition-colors"
				>
					{t("ui.refreshStatus")}
				</button>
			</div>

			{/* Settings Card */}
			<div className="bg-app-box border border-app-line rounded-lg p-4">
				<h2 className="text-lg font-medium text-ink mb-4">{t('daemon.settings')}</h2>

				<div className="space-y-4">
					{/* Auto-start Toggle */}
					<div className="flex items-center justify-between">
						<div>
							<h3 className="text-sm font-medium text-ink">{t('daemon.autoStart')}</h3>
							<p className="text-xs text-ink-dull mt-1">
								{t("ui.startDaemonAutomaticallyWhenAppLaunches")}
							</p>
						</div>
						<button
							onClick={() => setAutoStart(!autoStart)}
							className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
								autoStart ? 'bg-accent' : 'bg-app-line'
							}`}
						>
							<span
								className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
									autoStart ? 'translate-x-6' : 'translate-x-1'
								}`}
							/>
						</button>
					</div>

					{/* Run in Process Toggle */}
					<div className="flex items-center justify-between">
						<div>
							<h3 className="text-sm font-medium text-ink">{t('daemon.runInProcess')}</h3>
							<p className="text-xs text-ink-dull mt-1">
								{t("ui.runDaemonInTheAppProcess")}
							</p>
						</div>
						<button
							onClick={() => setRunInProcess(!runInProcess)}
							className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
								runInProcess ? 'bg-accent' : 'bg-app-line'
							}`}
						>
							<span
								className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
									runInProcess ? 'translate-x-6' : 'translate-x-1'
								}`}
							/>
						</button>
					</div>
				</div>
			</div>

			{/* macOS Background Permission Notice */}
			<div className="bg-sidebar-box border border-sidebar-line rounded-lg p-4">
				<div className="flex items-start gap-3">
					<Warning className="size-5 text-yellow-500 flex-shrink-0 mt-0.5" weight="fill" />
					<div>
						<h3 className="text-sm font-medium text-ink">{t('daemon.backgroundItems')}</h3>
						<p className="text-xs text-ink-dull mt-1">
							On macOS, running background processes requires permission. If the daemon fails to
							start automatically, check System Settings → General → Login Items & Extensions
							and allow WingDrive to run in the background.
						</p>
						<button
							onClick={handleOpenSettings}
							className="mt-2 text-xs text-accent hover:text-accent-deep font-medium"
						>
							Open System Settings →
						</button>
					</div>
				</div>
			</div>

			{/* Actions */}
			<div className="flex gap-3">
				<button
					onClick={handleStartDaemon}
					disabled={isRunning || isStarting || isLoading}
					className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-green-500 hover:bg-green-600 disabled:bg-app-line disabled:text-ink-faint text-white rounded-md text-sm font-medium transition-colors"
				>
					{isStarting ? (
						<CircleNotch className="size-4 animate-spin" weight="bold" />
					) : (
						<Power className="size-4" weight="bold" />
					)}
					{isStarting ? t('daemon.starting') : t('daemon.start')}
				</button>
				<button
					onClick={handleStopDaemon}
					disabled={!isRunning || isStopping || isLoading || !status?.started_by_us}
					className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-red-500 hover:bg-red-600 disabled:bg-app-line disabled:text-ink-faint text-white rounded-md text-sm font-medium transition-colors"
				>
					{isStopping ? (
						<CircleNotch className="size-4 animate-spin" weight="bold" />
					) : (
						<Power className="size-4" weight="bold" />
					)}
					{isStopping ? t('daemon.stopping') : t('daemon.stop')}
				</button>
			</div>
		</div>
	);
}
