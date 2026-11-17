import { LocalizedText as Text } from "../../i18n/LocalizedText";
import React, { useEffect, useState } from "react";
import { ScrollView, Alert, Linking } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { useTranslation } from "react-i18next";
import AsyncStorage from "@react-native-async-storage/async-storage";
import type { UpdateAppConfigInput, UpdateLibraryConfigInput } from "@wingdrive/ts-client/src/generated/types";
import {
	useCoreAction,
	useCoreQuery,
	useLibraryAction,
	useLibraryQuery,
} from "../../client";
import { useCurrentLibraryId } from "../../client/hooks/useQuery";
import { useAppReset } from "../../contexts";
import { chooseLanguage as storeLanguageChoice, loadLanguageChoice } from "../../i18n";
import type { LanguageChoice } from "../../i18n/language";
import { usePreferencesStore } from "../../stores/preferences";
import { SettingsGroup, SettingsLink, SettingsOption, SettingsToggle } from "../../components/primitive";
import { RenameDialog } from "../../components/RenameDialog";

const REPOSITORY_URL = "https://github.com/bernardopg/wingdrive";

export function SettingsScreen() {
	const { t } = useTranslation("common");
	const insets = useSafeAreaInsets();
	const { resetApp } = useAppReset();
	const libraryId = useCurrentLibraryId();
	const [renaming, setRenaming] = useState<"device" | "library" | null>(null);
	const [languageChoice, setLanguageChoice] = useState<LanguageChoice | null>(null);

	useEffect(() => {
		void loadLanguageChoice().then(setLanguageChoice);
	}, []);

	const status = useCoreQuery("core.status", {});
	const appConfig = useCoreQuery("config.app.get", null);
	const libraryConfig = useLibraryQuery("config.library.get", null);
	const libraryInfo = useLibraryQuery("libraries.info", null);

	const updateApp = useCoreAction("config.app.update", { onSuccess: () => appConfig.refetch() });
	const updateLibrary = useLibraryAction("config.library.update", {
		onSuccess: () => libraryConfig.refetch(),
	});
	const updateDevice = useCoreAction("device.update", { onSuccess: () => status.refetch() });
	const renameLibrary = useLibraryAction("libraries.rename", {
		onSuccess: () => libraryInfo.refetch(),
	});
	const clearCache = useCoreAction("core.ephemeral_reset");
	const resetData = useCoreAction("core.reset");

	const autoSwitchOnSync = usePreferencesStore((s) => s.autoSwitchOnSync);
	const setAutoSwitchOnSync = usePreferencesStore((s) => s.setAutoSwitchOnSync);

	const config = appConfig.data;
	const library = libraryConfig.data;
	const device = status.data?.device_info;
	const language = config?.preferences.language === "pt" ? "pt" : "en";
	// No stored choice means the app follows the phone, unless the core already says "pt".
	const shownChoice: LanguageChoice = languageChoice ?? (language === "pt" ? "pt" : "system");
	const languageLabel =
		shownChoice === "pt" ? "Português" : shownChoice === "en" ? "English" : t("System default");

	const setAppConfig = (input: UpdateAppConfigInput) =>
		updateApp.mutate(input, {
			onError: (error) => Alert.alert(t("Error"), error.message),
		});
	const setLibraryConfig = (input: UpdateLibraryConfigInput) =>
		updateLibrary.mutate(input, {
			onError: (error) => Alert.alert(t("Error"), error.message),
		});

	const chooseLanguage = () => {
		const pick = (next: LanguageChoice) => {
			setLanguageChoice(next);
			void storeLanguageChoice(next).then((shown) => setAppConfig({ language: shown }));
		};
		Alert.alert(t("Language"), undefined, [
			{ text: t("System default"), onPress: () => pick("system") },
			{ text: "English", onPress: () => pick("en") },
			{ text: "Português", onPress: () => pick("pt") },
			{ text: t("Cancel"), style: "cancel" },
		]);
	};

	const handleClearCache = () => {
		Alert.alert(t("Clear cache"), t("Folders browsed without indexing are listed again from disk."), [
			{ text: t("Cancel"), style: "cancel" },
			{
				text: t("Clear"),
				onPress: () =>
					clearCache.mutate(
						{ confirm: true },
						{ onSuccess: (result) => Alert.alert(t("Cache cleared"), result.message) },
					),
			},
		]);
	};

	const handleResetData = () => {
		Alert.alert(
			t("Reset All Data"),
			t("This will permanently delete all libraries, settings, and cached data. The app will refresh automatically. Are you sure?"),
			[
				{ text: t("Cancel"), style: "cancel" },
				{
					text: t("Reset"),
					style: "destructive",
					onPress: () =>
						resetData.mutate(
							{ confirm: true },
							{
								onSuccess: async () => {
									await AsyncStorage.clear();
									resetApp();
								},
								onError: (error) =>
									Alert.alert(t("Error"), error.message || t("Failed to reset data")),
							},
						),
				},
			],
		);
	};

	return (
		<ScrollView
			className="flex-1 bg-app"
			contentContainerStyle={{
				paddingTop: insets.top + 16,
				paddingBottom: insets.bottom + 100,
				paddingHorizontal: 16,
			}}
		>
			<Text className="text-2xl font-bold text-ink mb-6">Settings</Text>

			<SettingsGroup header="This Device" footer="Paired devices see this name.">
				<SettingsLink
					label="Device name"
					description={device?.name ?? "…"}
					onPress={() => device && setRenaming("device")}
				/>
				<SettingsOption
					label="System"
					value={[device?.os, device?.hardware_model].filter(Boolean).join(" · ") || "…"}
				/>
			</SettingsGroup>

			<SettingsGroup header="General">
				<SettingsOption
					label="Language"
					value={languageLabel}
					onPress={chooseLanguage}
				/>
				<SettingsToggle
					label="Switch to synced libraries"
					description="Open a library as soon as a paired device shares it"
					value={autoSwitchOnSync}
					onValueChange={setAutoSwitchOnSync}
				/>
			</SettingsGroup>

			{libraryId && (
				<SettingsGroup header="Library">
					<SettingsLink
						label="Library name"
						description={libraryInfo.data?.name ?? "…"}
						onPress={() => libraryInfo.data && setRenaming("library")}
					/>
					<SettingsToggle
						label="Generate thumbnails"
						value={library?.generate_thumbnails ?? false}
						onValueChange={(value) => setLibraryConfig({ generate_thumbnails: value })}
					/>
					<SettingsToggle
						label="Sync with paired devices"
						value={library?.sync_enabled ?? false}
						onValueChange={(value) => setLibraryConfig({ sync_enabled: value })}
					/>
					<SettingsToggle
						label="Track external volumes"
						description="Add USB drives and SD cards automatically"
						value={library?.auto_track_external_volumes ?? false}
						onValueChange={(value) => setLibraryConfig({ auto_track_external_volumes: value })}
					/>
				</SettingsGroup>
			)}

			<SettingsGroup header="Services" footer="Changes apply the next time the app starts.">
				<SettingsToggle
					label="Peer-to-peer networking"
					description="Pairing, sync and transfers with your devices"
					value={config?.services.networking_enabled ?? false}
					onValueChange={(value) => setAppConfig({ networking_enabled: value })}
				/>
				<SettingsToggle
					label="Watch for file changes"
					value={config?.services.fs_watcher_enabled ?? false}
					onValueChange={(value) => setAppConfig({ fs_watcher_enabled: value })}
				/>
				<SettingsToggle
					label="Monitor volumes"
					value={config?.services.volume_monitoring_enabled ?? false}
					onValueChange={(value) => setAppConfig({ volume_monitoring_enabled: value })}
				/>
			</SettingsGroup>

			<SettingsGroup header="Privacy">
				<SettingsToggle
					label="Share anonymous usage data"
					value={config?.telemetry_enabled ?? false}
					onValueChange={(value) => setAppConfig({ telemetry_enabled: value })}
				/>
			</SettingsGroup>

			<SettingsGroup header="Storage">
				<SettingsLink label="Clear cache" onPress={handleClearCache} />
				<SettingsLink label="Reset All Data" onPress={handleResetData} />
			</SettingsGroup>

			<SettingsGroup header="About">
				<SettingsOption label="Version" value={status.data?.version ?? "…"} />
				<SettingsLink label="Source code" onPress={() => Linking.openURL(REPOSITORY_URL)} />
				<SettingsLink
					label="Report a problem"
					onPress={() => Linking.openURL(`${REPOSITORY_URL}/issues/new`)}
				/>
			</SettingsGroup>

			{renaming === "device" && device && (
				<RenameDialog
					title={t("Device name")}
					initialValue={device.name}
					onClose={() => setRenaming(null)}
					onSubmit={(name) => {
						setRenaming(null);
						updateDevice.mutate({ name }, { onError: (error) => Alert.alert(t("Error"), error.message) });
					}}
				/>
			)}
			{renaming === "library" && libraryId && libraryInfo.data && (
				<RenameDialog
					title={t("Library name")}
					initialValue={libraryInfo.data.name}
					onClose={() => setRenaming(null)}
					onSubmit={(new_name) => {
						setRenaming(null);
						renameLibrary.mutate(
							{ library_id: libraryId, new_name },
							{ onError: (error) => Alert.alert(t("Error"), error.message) },
						);
					}}
				/>
			)}
		</ScrollView>
	);
}
