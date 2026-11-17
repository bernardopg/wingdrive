import { LocalizedText as Text } from "../../../i18n/LocalizedText";
import React from "react";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { View, FlatList, Pressable, Image } from "react-native";
import type { File } from "@wingdrive/ts-client";
import { getVirtualMetadata, isVirtualFile, getFileKindForIcon } from "@wingdrive/ts-client";
import { getIcon } from "@wingdrive/assets/util/mobile";
import { imageSource } from "../../../utils/imageSource";
import { useTranslation } from "react-i18next";
import { relativeDate } from "../fileLogic";

interface ListViewProps {
	files: File[];
	onFilePress: (file: File) => void;
	/** Long press: the file's actions */
	onFileLongPress?: (file: File) => void;
	/** Shown instead of "No items" when the listing is empty */
	emptyState?: React.ReactNode;
}

function formatBytes(bytes: number): string {
	if (bytes === 0) return "0 B";
	const k = 1024;
	const sizes = ["B", "KB", "MB", "GB", "TB"];
	const i = Math.floor(Math.log(bytes) / Math.log(k));
	return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

function FileRow({
	file,
	onPress,
	onLongPress,
}: {
	file: File;
	onPress: () => void;
	onLongPress?: () => void;
}) {
	const { t } = useTranslation("common");
	const virtualMetadata = getVirtualMetadata(file);
	const isVirtual = isVirtualFile(file);
	const modified = relativeDate(file.modified_at);

	// Get icon source
	const iconSource = (() => {
		// Virtual files have custom icons
		if (virtualMetadata?.iconUrl) {
			return virtualMetadata.iconUrl;
		}

		// Use getIcon from mobile-compatible util
		const kindForIcon = getFileKindForIcon(file);
		return getIcon(
			kindForIcon,
			true, // isDark - use dark mode icons for mobile
			file.extension,
			file.kind === "Directory",
		);
	})();

	return (
		<Pressable
			onPress={onPress}
			onLongPress={onLongPress}
			className="flex-row items-center px-4 py-3 border-b border-app-line active:bg-app-hover"
		>
			{/* Icon */}
			<Image
				source={imageSource(iconSource)}
				className="w-10 h-10 mr-3"
				style={{ resizeMode: "contain" }}
			/>

			{/* File info */}
			<View className="flex-1">
				<Text className="text-ink font-medium" numberOfLines={1}>
					{file.name}
				</Text>
				<View className="flex-row gap-2 mt-0.5">
					{!isVirtual && file.size > 0 && (
						<Text className="text-ink-faint text-xs">
							{formatBytes(file.size)}
						</Text>
					)}
					{!isVirtual && modified && (
						<>
							<Text className="text-ink-faint text-xs">•</Text>
							<Text className="text-ink-faint text-xs">
								{t(modified.key, { count: modified.count })}
							</Text>
						</>
					)}
					{isVirtual && virtualMetadata?.type && (
						<Text className="text-ink-faint text-xs capitalize">
							{virtualMetadata.type}
						</Text>
					)}
				</View>
			</View>

			{/* Chevron for directories */}
			{file.kind === "Directory" && (
				<Text className="text-ink-faint ml-2">›</Text>
			)}
		</Pressable>
	);
}

export function ListView({ files, onFilePress, onFileLongPress, emptyState }: ListViewProps) {
	// Edge-to-edge on Android: keep the last row above the navigation bar.
	const insets = useSafeAreaInsets();
	if (files.length === 0) {
		return (
			<View className="flex-1 items-center justify-center">
				{emptyState ?? <Text className="text-ink-dull">No items</Text>}
			</View>
		);
	}

	return (
		<FlatList
			data={files}
			keyExtractor={(item) => item.id}
			renderItem={({ item }) => (
				<FileRow
					file={item}
					onPress={() => onFilePress(item)}
					onLongPress={onFileLongPress && (() => onFileLongPress(item))}
				/>
			)}
			contentContainerStyle={{ paddingBottom: insets.bottom + 16 }}
			className="flex-1"
		/>
	);
}
