import { useCallback, useEffect, useState } from "react";
import { AppState, View } from "react-native";
import { WingMobileCore } from "wing-mobile-core";
import { LocalizedText as Text } from "../i18n/LocalizedText";
import { Button, Card } from "./primitive";

/**
 * Whether the app may read shared storage. Checked again each time the app
 * comes back to the foreground, since the grant happens in system settings.
 */
export function useStorageAccess() {
	const [granted, setGranted] = useState(() => WingMobileCore.hasAllFilesAccess());
	useEffect(() => {
		const subscription = AppState.addEventListener("change", (state) => {
			if (state === "active") setGranted(WingMobileCore.hasAllFilesAccess());
		});
		return () => subscription.remove();
	}, []);
	return granted;
}

/**
 * Asks for "All files access" while the app does not have it (MOB-017). Without
 * it the phone's own storage shows no files.
 */
export function StorageAccessCard() {
	const granted = useStorageAccess();
	const openSettings = useCallback(() => {
		WingMobileCore.openAllFilesAccessSettings();
	}, []);
	if (granted) return null;
	return (
		<Card className="mb-6">
			<Text className="text-ink text-base font-semibold mb-1">Allow access to files</Text>
			<Text className="text-ink-dull text-sm mb-3">
				WingDrive needs All files access to show and manage the files on this device.
			</Text>
			<View className="flex-row">
				<Button variant="accent" onPress={openSettings}>
					<Text className="text-white font-medium">Open settings</Text>
				</Button>
			</View>
		</Card>
	);
}
