import { LocalizedText as Text } from "../../../i18n/LocalizedText";
import React from "react";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { View, FlatList, Pressable, Text as NativeText } from "react-native";
import type { File } from "@wingdrive/ts-client";
import {
	getVirtualMetadata,
	isVirtualFile,
	fileDisplayName,
	fileDisplaySize,
} from "@wingdrive/ts-client";
import { FileIcon } from "../components/FileIcon";
import { useTranslation } from "react-i18next";
import { relativeDate } from "../fileLogic";

interface ListViewProps {
	files: File[];
	onFilePress: (file: File) => void;
	/** Long press: the file's actions */
	onFileLongPress?: (file: File) => void;
	/** The path of a file on this phone, for image and video previews */
	localPathOf?: (file: File) => string | null;
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
	localPath,
}: {
	file: File;
	onPress: () => void;
	onLongPress?: () => void;
	localPath: string | null;
}) {
	const { t } = useTranslation("common");
	const virtualMetadata = getVirtualMetadata(file);
	const isVirtual = isVirtualFile(file);
	const modified = relativeDate(file.modified_at);
	// A folder's own metadata size is not its contents; show it only once measured.
	const size = fileDisplaySize(file);

	return (
		<Pressable
			onPress={onPress}
			onLongPress={onLongPress}
			className="flex-row items-center px-4 py-3 border-b border-app-line active:bg-app-hover"
		>
			{/* Icon */}
			<View className="mr-3">
				<FileIcon file={file} localPath={localPath} size={40} />
			</View>

			{/* File info */}
			<View className="flex-1">
				{/* User data, never looked up in the catalog ("Notifications" stays as is) */}
				<NativeText className="text-ink font-medium" numberOfLines={1}>
					{fileDisplayName(file)}
				</NativeText>
				<View className="flex-row gap-2 mt-0.5">
					{!isVirtual && !!size && (
						<Text className="text-ink-faint text-xs">
							{formatBytes(size)}
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

export function ListView({
	files,
	onFilePress,
	onFileLongPress,
	localPathOf,
	emptyState,
}: ListViewProps) {
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
					localPath={localPathOf?.(item) ?? null}
				/>
			)}
			contentContainerStyle={{ paddingBottom: insets.bottom + 16 }}
			className="flex-1"
		/>
	);
}
