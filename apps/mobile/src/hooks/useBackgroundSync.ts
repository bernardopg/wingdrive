import { useCallback, useEffect, useState } from "react";
import { AppState, Linking, PermissionsAndroid, Platform } from "react-native";
import { WingMobileCore } from "wing-mobile-core";
import { needsNotificationPrompt } from "./backgroundSync";

/**
 * Background sync on Android (MOB-018): a foreground service keeps the core syncing with
 * paired devices while the app is closed. Read again when the app returns to the
 * foreground, since battery and notification grants happen in system settings.
 */
export function useBackgroundSync() {
	const [state, setState] = useState(() => WingMobileCore.getBackgroundSync());
	const refresh = useCallback(() => setState(WingMobileCore.getBackgroundSync()), []);

	useEffect(() => {
		const subscription = AppState.addEventListener("change", (next) => {
			if (next === "active") refresh();
		});
		return () => subscription.remove();
	}, [refresh]);

	const setEnabled = useCallback(
		async (enabled: boolean) => {
			if (enabled && needsNotificationPrompt(Platform.OS, Platform.Version)) {
				// The service runs without it; Android only hides the notification.
				await PermissionsAndroid.request(PermissionsAndroid.PERMISSIONS.POST_NOTIFICATIONS);
			}
			WingMobileCore.setBackgroundSync(enabled);
			refresh();
		},
		[refresh],
	);

	const allowUnrestrictedBattery = useCallback(() => {
		WingMobileCore.openBatteryOptimizationSettings();
	}, []);

	const allowNotifications = useCallback(() => {
		void Linking.openSettings();
	}, []);

	return { state, setEnabled, allowUnrestrictedBattery, allowNotifications };
}
