import React from "react";
import { Alert, Image } from "react-native";
import { useRouter } from "expo-router";
import { useLibraryAction, useNormalizedQuery } from "../../../client";
import { SettingsGroup, SettingsLink } from "../../../components/primitive";
import { useTranslation } from "react-i18next";
import FolderIcon from "@wingdrive/assets/icons/Folder.webp";

export function LocationsGroup() {
	const router = useRouter();
	const { t } = useTranslation("common");
	const removeLocation = useLibraryAction("locations.remove", {
		onError: (error) => Alert.alert(t("Error"), error.message),
	});
	const { data: locationsData } = useNormalizedQuery({
		query: "locations.list",
		input: null,
		resourceType: "location",
	});

	const { data: devices } = useNormalizedQuery({
		query: "devices.list",
		input: {
			include_offline: true,
			include_details: false,
			show_paired: true,
		},
		resourceType: "device",
	});

	const locations = locationsData?.locations ?? [];

	if (locations.length === 0) {
		return null;
	}

	// Helper to get device name from device_slug
	const getDeviceName = (location: any) => {
		const deviceSlug = location.wing_path?.Physical?.device_slug;
		if (!deviceSlug) return "Unknown device";
		const device = devices?.find((d) => d.slug === deviceSlug);
		return device?.name || "Unknown device";
	};

	return (
		<SettingsGroup header="Locations">
			{locations.map((location: any) => (
				<SettingsLink
					key={location.id}
					icon={
						<Image
							source={FolderIcon}
							className="w-8 h-8"
							style={{ resizeMode: "contain" }}
						/>
					}
					label={location.name || "Unnamed"}
					description={getDeviceName(location)}
					onLongPress={() =>
						// Removing a location forgets its index; the files stay on the device.
						Alert.alert(
							t("Remove location?"),
							`${location.name}\n\n${t("The files stay where they are. WingDrive stops indexing and watching them.")}`,
							[
								{ text: t("Cancel"), style: "cancel" },
								{
									text: t("Remove"),
									style: "destructive",
									onPress: () => removeLocation.mutate({ location_id: location.id }),
								},
							],
						)
					}
					onPress={() => {
						router.push({
							pathname: "/explorer",
							params: {
								type: "path",
								path: JSON.stringify(location.wing_path),
							},
						});
					}}
				/>
			))}
		</SettingsGroup>
	);
}
