import type { BackgroundSyncState } from "wing-mobile-core";

export type BackgroundSyncHint = "battery" | "notifications";

/**
 * What the user still has to allow for background sync to hold (MOB-018). Battery
 * optimization lets some phones stop the service; without notifications Android hides
 * the notice that the app is running. Nothing to show while background sync is off.
 */
export function backgroundSyncHints(state: BackgroundSyncState): BackgroundSyncHint[] {
	if (!state.supported || !state.enabled) return [];
	const hints: BackgroundSyncHint[] = [];
	if (!state.unrestrictedBattery) hints.push("battery");
	if (!state.notificationsAllowed) hints.push("notifications");
	return hints;
}

/** Android 13 (API 33) made the notification permission a runtime prompt. */
export function needsNotificationPrompt(platform: string, apiLevel: number | string): boolean {
	return platform === "android" && Number(apiLevel) >= 33;
}
