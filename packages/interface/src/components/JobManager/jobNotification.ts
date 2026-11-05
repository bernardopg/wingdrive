/**
 * Desktop notifications for jobs that end while WingDrive is in the background.
 *
 * Only file operations the user started (copy, move, delete, ...) notify:
 * indexing and thumbnail jobs run all the time and would flood the
 * notification center. The window being focused also suppresses them, since
 * the job list already shows the outcome.
 */

/** Matches `NotificationKind` in apps/tauri/src-tauri/src/notifications.rs. */
export type DesktopNotificationKind = 'transferComplete' | 'transferError' | 'info';

/** Matches `NotificationRequest` in apps/tauri/src-tauri/src/notifications.rs. */
export interface DesktopNotificationRequest {
	title: string;
	body: string;
	kind: DesktopNotificationKind;
	/** Label of the action that brings WingDrive to the front. */
	openLabel?: string;
}

export type JobOutcome = 'completed' | 'failed';

type Translate = (key: string) => string;

/** True when a job's end deserves a desktop notification. */
export function shouldNotifyForJob(
	actionType: string | null | undefined,
	windowFocused: boolean
): boolean {
	if (windowFocused) return false;
	return typeof actionType === 'string' && actionType.startsWith('files.');
}

/** Builds the notification for a job that ended with `outcome`. */
export function buildJobNotification(
	outcome: JobOutcome,
	jobName: string,
	t: Translate
): DesktopNotificationRequest {
	return {
		title: t(
			outcome === 'completed'
				? 'dialogs:jobs.notificationCompleted'
				: 'dialogs:jobs.jobFailed'
		),
		body: jobName,
		kind: outcome === 'completed' ? 'transferComplete' : 'transferError',
		openLabel: t('dialogs:jobs.notificationOpen')
	};
}

/** Whether the WingDrive window has the user's attention right now. */
export function isWindowFocused(): boolean {
	if (typeof document === 'undefined') return true;
	return !document.hidden && document.hasFocus();
}
