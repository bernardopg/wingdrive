import React from "react";
import { Image } from "react-native";
import { useRouter } from "expo-router";
import { useNormalizedQuery } from "../../../client";
import { getVolumeIcon } from "@wingdrive/ts-client";
import { SettingsGroup, SettingsLink } from "../../../components/primitive";
import { imageSource } from "../../../utils/imageSource";

export function VolumesGroup() {
	const router = useRouter();
	const { data: volumesData } = useNormalizedQuery(
		{
			query: "volumes.list",
			input: { filter: "All" },
			resourceType: "volume",
		}
	);

	const { data: devices } = useNormalizedQuery({
		query: "devices.list",
		input: {
			include_offline: true,
			include_details: false,
			show_paired: true,
		},
		resourceType: "device",
	});

	const volumes = volumesData?.volumes || [];

	if (volumes.length === 0) {
		return null;
	}

	return (
		<SettingsGroup header="Volumes">
			{volumes.map((volume) => {
				const volumeIconSrc = getVolumeIcon(volume);
				const device = devices?.find((d) => d.id === volume.device_id);

				return (
					<SettingsLink
						key={volume.id}
						icon={
							<Image
								source={imageSource(volumeIconSrc)}
								className="w-8 h-8"
								style={{ resizeMode: "contain" }}
							/>
						}
						label={volume.display_name || volume.name}
						description={
							volume.is_tracked ? "Tracked" : "Not tracked"
						}
						onPress={() => {
							if (device) {
								const sdPath = {
									Physical: {
										device_slug: device.slug,
										path: volume.mount_point || "/",
									},
								};
								router.push({
									pathname: "/explorer",
									params: {
										type: "path",
										path: JSON.stringify(sdPath),
									},
								});
							}
						}}
					/>
				);
			})}
		</SettingsGroup>
	);
}
