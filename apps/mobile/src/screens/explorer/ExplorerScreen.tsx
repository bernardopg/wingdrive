import { LocalizedText as Text } from "../../i18n/LocalizedText";
import React, { useState, useMemo } from "react";
import { View, ActivityIndicator } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { useLocalSearchParams, useRouter } from "expo-router";
import { useExplorerFiles } from "./hooks/useExplorerFiles";
import { useFileActions } from "./hooks/useFileActions";
import { ListView } from "./views/ListView";
import { GridView } from "./views/GridView";
import { useNormalizedQuery } from "../../client";
import { GlassButton } from "../../components/GlassButton";
import { GlassContextMenu } from "../../components/GlassContextMenu";
import { SearchToolbar } from "../../components/SearchToolbar";
import { StorageAccessCard } from "../../components/StorageAccessCard";
import { useSearchStore } from "./context/SearchContext";
import { List, ArrowLeft, FolderPlus, MapPin } from "phosphor-react-native";
import type { File, WingPath } from "@wingdrive/ts-client";

type ViewMode = "list" | "grid";

export function ExplorerScreen() {
	const insets = useSafeAreaInsets();
	const router = useRouter();
	const searchParams = useLocalSearchParams<{
		type?: string;
		path?: string;
		view?: string;
		id?: string;
		title?: string;
	}>();
	const [viewMode, setViewMode] = useState<ViewMode>("list");
	const { isSearchMode, query } = useSearchStore();

	// Parse params into the format expected by hooks
	const params = useMemo(() => {
		if (searchParams.type === "path" && searchParams.path) {
			return { type: "path" as const, path: searchParams.path };
		}
		if (searchParams.type === "view" && searchParams.view) {
			return {
				type: "view" as const,
				view: searchParams.view,
				id: searchParams.id,
			};
		}
		return undefined;
	}, [searchParams]);

	// Fetch files
	const { files, isLoading } = useExplorerFiles(params);
	const fileActions = useFileActions();
	const currentFolder = useMemo((): WingPath | null => {
		if (params?.type !== "path") return null;
		try {
			return JSON.parse(params.path) as WingPath;
		} catch {
			return null;
		}
	}, [params]);
	const handleFileLongPress = (file: File) => {
		if (fileActions.hasActions(file)) fileActions.showActions(file);
	};

	// Devices listed in the library, to explain an empty device view.
	const { data: libraryDevices } = useNormalizedQuery({
		query: "devices.list",
		input: { include_offline: true, include_details: false, show_paired: true },
		resourceType: "device",
		enabled: params?.type === "view" && params.view === "device",
	});
	const viewedDevice =
		params?.type === "view" && params.view === "device"
			? libraryDevices?.find((device) => device.id === params.id)
			: undefined;
	// An empty device view means missing storage access on this phone, or a
	// device that shares no folders with this library (MOB-017).
	const libraryViewEmpty =
		params?.type === "view" && params.view === "favorites" ? (
			<Text className="text-ink-dull text-center px-8">
				Files marked as favorite on any device of this library show up here.
			</Text>
		) : params?.type === "view" && params.view === "recents" ? (
			<Text className="text-ink-dull text-center px-8">
				Files indexed recently in this library show up here.
			</Text>
		) : undefined;
	const emptyState = libraryViewEmpty ?? (viewedDevice ? (
		viewedDevice.is_current ? (
			<View className="px-6 w-full">
				<StorageAccessCard />
				<Text className="text-ink-dull text-center">No items</Text>
			</View>
		) : (
			<Text className="text-ink-dull text-center px-8">
				This device shares no folders with this library yet. Add a location on it to browse its files here.
			</Text>
		)
	) : undefined);

	// Fetch device for path display
	const { data: devices } = useNormalizedQuery({
		query: "devices.list",
		input: {
			include_offline: true,
			include_details: false,
			show_paired: true,
		},
		resourceType: "device",
		enabled: params?.type === "path",
	});

	// Get title for header
	const title = (() => {
		if (params?.type === "view") {
			if (params.view === "devices") return "Devices";
			if (params.view === "recents") return "Recents";
			if (params.view === "favorites") return "Favorites";
			if (params.view === "device" && params.id) {
				const device = devices?.find((d) => d.id === params.id);
				return device?.name || "Device";
			}
		}

		if (params?.type === "path") {
			// Set by the screen that opened this folder: a volume's name rather than the last
			// segment of its mount point, which is "0" for /storage/emulated/0 (MOB-017).
			if (searchParams.title) return searchParams.title;
			try {
				const wdPath = JSON.parse(params.path);
				if (wdPath.Physical) {
					const device = devices?.find(
						(d) => d.slug === wdPath.Physical.device_slug,
					);
					const pathParts = wdPath.Physical.path.split("/").filter(Boolean);
					if (pathParts.length === 0) {
						return device?.name || "Root";
					}
					return pathParts[pathParts.length - 1];
				}
			} catch (e) {
				console.error("[ExplorerScreen] Failed to parse path:", e);
			}
		}

		return "Explorer";
	})();

	const handleFilePress = (file: File) => {
		// If it's a directory, navigate into it
		if (file.kind === "Directory") {
			router.push({
				pathname: "/explorer",
				params: {
					type: "path",
					path: JSON.stringify(file.wing_path),
					title: file.name,
				},
			});
			return;
		}
		// Files of this phone open in the app the user picks; there is no built-in preview yet.
		if (!fileActions.openIfLocal(file) && fileActions.hasActions(file)) {
			fileActions.showActions(file);
		}
	};

	return (
		<View className="flex-1 bg-app">
			{/* Header */}
			<View
				className="bg-app-box border-b border-app-line"
				style={{ paddingTop: insets.top }}
			>
				<View className="relative flex-row items-center justify-between px-4 h-14">
					{/* Back button */}
					<GlassButton
						onPress={() => router.back()}
						icon={
							<ArrowLeft size={20} color="hsl(235, 10%, 55%)" weight="bold" />
						}
					/>

					{/* Title - absolutely centered */}
					<Text
						className="absolute left-0 right-0 text-ink font-semibold text-lg text-center"
						pointerEvents="none"
					>
						{title}
					</Text>

					{/* View mode menu */}
					<GlassContextMenu
						items={[
							...(currentFolder && fileActions.canCreateIn(currentFolder)
								? [
										{
											label: "New Folder",
											onPress: () => fileActions.newFolder(currentFolder),
											icon: <FolderPlus size={20} color="hsl(235, 10%, 55%)" weight="bold" />,
										},
									]
								: []),
							...(currentFolder && fileActions.canAddLocation(currentFolder)
								? [
										{
											label: "Add as Location",
											onPress: () => fileActions.addAsLocation(currentFolder, title),
											icon: <MapPin size={20} color="hsl(235, 10%, 55%)" weight="bold" />,
										},
									]
								: []),
							{
								label: "List View",
								onPress: () => setViewMode("list"),
								icon: (
									<List
										size={20}
										color={
											viewMode === "list"
												? "hsl(208, 100%, 57%)"
												: "hsl(235, 10%, 55%)"
										}
										weight="bold"
									/>
								),
								active: viewMode === "list",
							},
							{
								label: "Grid View",
								onPress: () => setViewMode("grid"),
								icon: (
									<Text
										style={{
											fontSize: 20,
											color:
												viewMode === "grid"
													? "hsl(208, 100%, 57%)"
													: "hsl(235, 10%, 55%)",
										}}
									>
										⊞
									</Text>
								),
								active: viewMode === "grid",
							},
						]}
						trigger={
							<GlassButton
								icon={
									<Text className="text-ink-dull" style={{ fontSize: 18 }}>
										⋯
									</Text>
								}
							/>
						}
					/>
				</View>

				{/* Search Toolbar */}
				{isSearchMode && query.length >= 2 && <SearchToolbar />}
			</View>

			{/* Content */}
			{isLoading ? (
				<View className="flex-1 items-center justify-center">
					<ActivityIndicator size="large" color="hsl(208, 100%, 57%)" />
				</View>
			) : (
				<>
					{viewMode === "list" ? (
						<ListView
							files={files}
							onFilePress={handleFilePress}
							onFileLongPress={handleFileLongPress}
							emptyState={emptyState}
						/>
					) : (
						<GridView
							files={files}
							onFilePress={handleFilePress}
							onFileLongPress={handleFileLongPress}
							emptyState={emptyState}
						/>
					)}
				</>
			)}
			{fileActions.element}
		</View>
	);
}
