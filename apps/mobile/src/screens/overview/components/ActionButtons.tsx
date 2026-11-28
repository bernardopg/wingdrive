import React from "react";
import { View, Image } from "react-native";
import MobileIcon from "@wingdrive/assets/icons/Mobile.webp";
import NewLocationIcon from "@wingdrive/assets/icons/NewLocation.webp";
import { SettingsGroup, SettingsLink } from "../../../components/primitive";

interface ActionButtonsProps {
	onPairDevice: () => void;
	onAddStorage: () => void;
}

export function ActionButtons({
	onPairDevice,
	onAddStorage,
}: ActionButtonsProps) {
	return (
		<View className="mb-6">
			<SettingsGroup header="Quick Actions">
				<SettingsLink
					icon={
						<Image
							source={MobileIcon}
							className="w-6 h-6"
							style={{ resizeMode: "contain" }}
						/>
					}
					label="Pair New Device"
					description="Connect another device to share files"
					onPress={onPairDevice}
				/>
				<SettingsLink
					icon={
						<Image
							source={NewLocationIcon}
							className="w-6 h-6"
							style={{ resizeMode: "contain" }}
						/>
					}
					label="Add Storage"
					description="Open a folder and choose Add as Location"
					onPress={onAddStorage}
				/>
			</SettingsGroup>
		</View>
	);
}
