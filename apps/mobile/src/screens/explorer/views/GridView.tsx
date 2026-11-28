import { LocalizedText as Text } from "../../../i18n/LocalizedText";
import React from "react";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { View, FlatList, Pressable, Dimensions, Text as NativeText } from "react-native";
import type { File } from "@wingdrive/ts-client";
import { fileDisplayName, fileDisplaySize, getVirtualMetadata } from "@wingdrive/ts-client";
import { useTranslation } from "react-i18next";
import { rowDisplayName } from "../fileLogic";
import { FileIcon } from "../components/FileIcon";

interface GridViewProps {
	files: File[];
	onFilePress: (file: File) => void;
	/** Long press: the file's actions */
	onFileLongPress?: (file: File) => void;
	/** The path of a file on this phone, for image and video previews */
	localPathOf?: (file: File) => string | null;
	/** Shown instead of "No items" when the listing is empty */
	emptyState?: React.ReactNode;
}

const SCREEN_WIDTH = Dimensions.get("window").width;
const NUM_COLUMNS = 3;
const ITEM_PADDING = 8;
const CONTAINER_PADDING = 16;
const ITEM_WIDTH =
	(SCREEN_WIDTH - CONTAINER_PADDING * 2 - ITEM_PADDING * (NUM_COLUMNS - 1)) /
	NUM_COLUMNS;

function formatBytes(bytes: number | bigint | null): string {
	if (bytes === null) return "0 B";
	const numBytes = typeof bytes === "bigint" ? Number(bytes) : bytes;
	if (numBytes === 0) return "0 B";
	const k = 1024;
	const sizes = ["B", "KB", "MB", "GB", "TB"];
	const i = Math.floor(Math.log(numBytes) / Math.log(k));
	return Math.round(numBytes / Math.pow(k, i)) + " " + sizes[i];
}

function FileCard({
	file,
	onPress,
	onLongPress,
	localPath,
}: {
	file: File;
	onPress: () => void;
	onLongPress?: () => void;
	localPath: string | null;
}) {
	const { t, i18n } = useTranslation("common");
	const size = fileDisplaySize(file);

	return (
		<Pressable
			onPress={onPress}
			onLongPress={onLongPress}
			style={{ width: ITEM_WIDTH }}
			className="mb-4 active:opacity-70"
		>
			{/* Icon container - transparent bg matching web FileCard */}
			<View className="rounded-lg p-2 items-center justify-center aspect-square mb-2">
				<FileIcon file={file} localPath={localPath} size={80} />
			</View>

			{/* File name: user data, never looked up in the catalog */}
			<NativeText className="text-ink text-xs text-center" numberOfLines={2}>
				{rowDisplayName(fileDisplayName(file), getVirtualMetadata(file)?.type, (key) =>
					i18n.exists(key) ? t(key) : null,
				)}
			</NativeText>

			{/* File size; a folder's only once its contents were measured */}
			{!!size && (
				<Text className="text-ink-dull text-[10px] text-center mt-0.5">
					{formatBytes(size)}
				</Text>
			)}
		</Pressable>
	);
}

export function GridView({
	files,
	onFilePress,
	onFileLongPress,
	localPathOf,
	emptyState,
}: GridViewProps) {
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
				<FileCard
					file={item}
					onPress={() => onFilePress(item)}
					onLongPress={onFileLongPress && (() => onFileLongPress(item))}
					localPath={localPathOf?.(item) ?? null}
				/>
			)}
			numColumns={NUM_COLUMNS}
			columnWrapperStyle={{
				paddingHorizontal: CONTAINER_PADDING,
				justifyContent: "space-between",
			}}
			contentContainerStyle={{
				paddingTop: 16,
				paddingBottom: insets.bottom + 32,
			}}
			className="flex-1"
		/>
	);
}
