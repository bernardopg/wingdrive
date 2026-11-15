import { LocalizedText as Text } from "../../../i18n/LocalizedText";
import React, { useState } from "react";
import { View, Pressable } from "react-native";
import type { WingGroup, WingItem } from "@wingdrive/ts-client";
import { SettingsGroup } from "../../../components/primitive";
import { WingItem as WingItemComponent } from "./WingItem";
import { DevicesGroup } from "./DevicesGroup";
import { LocationsGroup } from "./LocationsGroup";
import { VolumesGroup } from "./VolumesGroup";
import { CaretDown, CaretRight } from "phosphor-react-native";

interface WingGroupProps {
	group: WingGroup;
	items: WingItem[];
}

export function WingGroupComponent({ group, items }: WingGroupProps) {
	const [isCollapsed, setIsCollapsed] = useState(group.is_collapsed ?? false);

	const handleToggle = () => {
		setIsCollapsed(!isCollapsed);
	};

	// System groups - use existing components
	if (group.group_type === "Devices") {
		return <DevicesGroup />;
	}

	if (group.group_type === "Locations") {
		return <LocationsGroup />;
	}

	if (group.group_type === "Volumes") {
		return <VolumesGroup />;
	}

	if (group.group_type === "Tags") {
		// Tags group not implemented yet
		return null;
	}

	// Custom/QuickAccess groups - render items
	return (
		<View className="mb-6">
			{/* Group Header */}
			<Pressable
				onPress={handleToggle}
				className="flex-row items-center justify-between px-4 mb-2"
			>
				<Text className="text-xs font-semibold text-ink-dull uppercase tracking-wider">
					{group.name}
				</Text>
				{isCollapsed ? (
					<CaretRight size={16} color="hsl(235, 10%, 55%)" weight="bold" />
				) : (
					<CaretDown size={16} color="hsl(235, 10%, 55%)" weight="bold" />
				)}
			</Pressable>

			{/* Group Items */}
			{!isCollapsed && items.length > 0 && (
				<SettingsGroup>
					{items
						.filter((item) => item.item_type !== "Overview")
						.map((item) => (
							<WingItemComponent key={item.id} item={item} />
						))}
				</SettingsGroup>
			)}
		</View>
	);
}
