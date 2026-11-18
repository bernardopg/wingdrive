import { useCallback, useState } from "react";
import { Alert, Platform } from "react-native";
import { useIsFocused } from "expo-router";
import { useTranslation } from "react-i18next";
import { useQueryClient } from "@tanstack/react-query";
import type { File, WingPath } from "@wingdrive/ts-client";
import { fileDisplayName, isVirtualFile, useJobs } from "@wingdrive/ts-client";
import { WingMobileCore } from "wing-mobile-core";
import {
	ArrowSquareOut,
	ArrowsLeftRight,
	Copy,
	PencilSimple,
	ShareNetwork,
	Tag,
	Trash,
} from "phosphor-react-native";
import { useLibraryAction, useNormalizedQuery } from "../../../client";
import { RenameDialog } from "../../../components/RenameDialog";
import { useFileClipboard, type ClipboardMode } from "../../../stores/fileClipboard";
import { FileActionsSheet, type FileAction } from "../components/FileActionsSheet";
import { FileMetadataSheet } from "../components/FileMetadataSheet";
import { isInLocation, localPath, pasteBlocker } from "../fileLogic";

const HAS_TRASH = Platform.OS === "android";
const ICON = "hsl(235, 10%, 55%)";

type Prompt = { kind: "rename"; file: File } | { kind: "newFolder"; parent: WingPath };

/**
 * Explorer file operations (MOB-007): open, share, copy, move, rename and delete through a long
 * press, and a new folder from the header menu. Copy and move hold the file until the user pastes
 * it in another folder. Listings refetch when an operation's job ends.
 */
export function useFileActions() {
	const { t } = useTranslation("common");
	const queryClient = useQueryClient();
	const focused = useIsFocused();
	const [selected, setSelected] = useState<File | null>(null);
	const [metadataFile, setMetadataFile] = useState<File | null>(null);
	const [prompt, setPrompt] = useState<Prompt | null>(null);

	const { data: devices } = useNormalizedQuery({
		query: "devices.list",
		input: { include_offline: true, include_details: false, show_paired: true },
		resourceType: "device",
	});
	const currentDeviceSlug = devices?.find((device) => device.is_current)?.slug;

	const invalidateListings = useCallback(() => {
		void queryClient.invalidateQueries({ queryKey: ["query:files.directory_listing"] });
	}, [queryClient]);

	// A failed or cancelled job can also leave changes on disk.
	useJobs({
		onJobCompleted: (_id, _type, output) => {
			invalidateListings();
			if (output?.type === "FileDelete") {
				void queryClient.invalidateQueries({ queryKey: ["core", "trash.list"] });
				void queryClient.invalidateQueries({ queryKey: ["query:search.files"] });
				if (focused && output.data.failed_count > 0) {
					Alert.alert(t("Error"), t(HAS_TRASH ? "Could not move the item to Trash" : "Could not delete the item"));
				}
			}
		},
		onJobFailed: invalidateListings,
		onJobCancelled: invalidateListings,
	});

	const showError = useCallback(
		(error: Error) => Alert.alert(t("Error"), error.message),
		[t],
	);
	const rename = useLibraryAction("files.rename", { onError: showError });
	const remove = useLibraryAction("files.delete", { onError: showError });
	const { data: locationsData } = useNormalizedQuery({
		query: "locations.list",
		input: null,
		resourceType: "location",
	});
	const addLocation = useLibraryAction("locations.add", {
		onError: showError,
		onSuccess: (_output, input) => {
			void queryClient.invalidateQueries({ queryKey: ["query:locations.list"] });
			invalidateListings();
			Alert.alert(t("Location added"), input.name ?? undefined);
		},
	});
	const clipboard = useFileClipboard();
	const copy = useLibraryAction("files.copy", {
		onError: showError,
		onSettled: (_receipt, error, input) => {
			useFileClipboard.getState().finishPaste(input.sources.paths, !error);
		},
	});
	const createFolder = useLibraryAction("files.createFolder", {
		onError: showError,
		onSuccess: invalidateListings,
	});

	const openFile = useCallback(
		(file: File, path: string) => {
			try {
				if (!WingMobileCore.openFile(path)) {
					Alert.alert(t("No app can open this file"), fileDisplayName(file));
				}
			} catch (error) {
				showError(error as Error);
			}
		},
		[showError, t],
	);

	const confirmDelete = useCallback(
		(file: File) => {
			const name = fileDisplayName(file);
			const message = HAS_TRASH ? name : `${name}\n\n${t("This cannot be undone.")}`;
			Alert.alert(t(HAS_TRASH ? "Move to Trash?" : "Delete permanently?"), message, [
				{ text: t("Cancel"), style: "cancel" },
				{
					text: t(HAS_TRASH ? "Move to Trash" : "Delete"),
					style: "destructive",
					onPress: () =>
						remove.mutate({
							targets: { paths: [file.wing_path] },
							permanent: !HAS_TRASH,
							recursive: file.kind === "Directory",
						}),
				},
			]);
		},
		[remove, t],
	);

	const actionsFor = useCallback(
		(file: File): FileAction[] => {
			const path = localPath(file.wing_path, currentDeviceSlug);
			if (!path) return [];
			const actions: FileAction[] = [];
			const indexed = locationsData?.locations.some((location) => {
				const root = localPath(location.wing_path, currentDeviceSlug);
				return location.index_mode !== "None" && root != null && isInLocation(path, [root]);
			});
			if (indexed && !file.archive) {
				actions.push({
					label: "Favorites and tags",
					icon: <Tag size={22} color={ICON} weight="bold" />,
					onPress: () => setMetadataFile(file),
				});
			}
			if (file.kind !== "Directory" && WingMobileCore.canOpenFiles) {
				actions.push(
					{
						label: "Open with",
						icon: <ArrowSquareOut size={22} color={ICON} weight="bold" />,
						onPress: () => openFile(file, path),
					},
					{
						label: "Share",
						icon: <ShareNetwork size={22} color={ICON} weight="bold" />,
						onPress: () => {
							try {
								WingMobileCore.shareFile(path);
							} catch (error) {
								showError(error as Error);
							}
						},
					},
				);
			}
			if (file.archive) return actions;
			const hold = (mode: ClipboardMode) =>
				clipboard.set(mode, [
					{
						path: file.wing_path,
						localPath: path,
						name: fileDisplayName(file),
						isDirectory: file.kind === "Directory",
					},
				]);
			actions.push(
				{
					label: "Copy",
					icon: <Copy size={22} color={ICON} weight="bold" />,
					onPress: () => hold("copy"),
				},
				{
					label: "Move",
					icon: <ArrowsLeftRight size={22} color={ICON} weight="bold" />,
					onPress: () => hold("move"),
				},
				{
					label: "Rename",
					icon: <PencilSimple size={22} color={ICON} weight="bold" />,
					onPress: () => setPrompt({ kind: "rename", file }),
				},
				{
					label: HAS_TRASH ? "Move to Trash" : "Delete",
					icon: <Trash size={22} color="#ef4444" weight="bold" />,
					onPress: () => confirmDelete(file),
					destructive: true,
				},
			);
			return actions;
		},
		[clipboard, confirmDelete, currentDeviceSlug, locationsData, openFile, showError],
	);

	/** Whether a long press on this file has anything to offer. */
	const hasActions = useCallback(
		(file: File) => !isVirtualFile(file) && actionsFor(file).length > 0,
		[actionsFor],
	);

	/** Opens a file of this phone in another app; false for folders and files elsewhere. */
	const openIfLocal = useCallback(
		(file: File) => {
			const path = localPath(file.wing_path, currentDeviceSlug);
			if (!path || file.kind === "Directory" || !WingMobileCore.canOpenFiles) return false;
			openFile(file, path);
			return true;
		},
		[currentDeviceSlug, openFile],
	);

	/** Whether a folder can receive a new folder: only folders on this phone. */
	const canCreateIn = useCallback(
		(parent: WingPath | null) => !!parent && !!localPath(parent, currentDeviceSlug),
		[currentDeviceSlug],
	);

	/** Whether a folder of this phone can become a location: neither one already nor inside one. */
	const canAddLocation = useCallback(
		(folder: WingPath | null) => {
			const path = folder ? localPath(folder, currentDeviceSlug) : null;
			if (!path) return false;
			const roots = (locationsData?.locations ?? [])
				.map((location) => localPath(location.wing_path, currentDeviceSlug))
				.filter((root): root is string => !!root);
			return !isInLocation(path, roots);
		},
		[currentDeviceSlug, locationsData],
	);

	/** Indexes a folder of this phone as a location, as the desktop's "Add Location" does (MOB-008). */
	const addAsLocation = useCallback(
		(folder: WingPath, name: string) =>
			addLocation.mutate({ path: folder, name, mode: "Deep", job_policies: null }),
		[addLocation],
	);

	const element = (
		<>
			{metadataFile && <FileMetadataSheet file={metadataFile} onClose={() => setMetadataFile(null)} />}
			{selected && (
				<FileActionsSheet
					file={selected}
					actions={actionsFor(selected)}
					onClose={() => setSelected(null)}
				/>
			)}
			{prompt?.kind === "rename" && (
				<RenameDialog
					title={t("Rename")}
					// The full name: `name` alone has no extension, and renaming to it dropped
					// the extension from the file on disk.
					initialValue={fileDisplayName(prompt.file)}
					selectUntil={prompt.file.kind === "Directory" ? undefined : prompt.file.name.length}
					onClose={() => setPrompt(null)}
					onSubmit={(name) => {
						setPrompt(null);
						rename.mutate({ target: prompt.file.wing_path, new_name: name });
					}}
				/>
			)}
			{prompt?.kind === "newFolder" && (
				<RenameDialog
					title={t("New Folder")}
					initialValue=""
					onClose={() => setPrompt(null)}
					onSubmit={(name) => {
						setPrompt(null);
						createFolder.mutate({ parent: prompt.parent, name });
					}}
				/>
			)}
		</>
	);

	/** Why the held files cannot go into `folder`, or null when they can be pasted there. */
	const pasteBlockerIn = useCallback(
		(folder: WingPath | null) => {
			const destination = folder ? localPath(folder, currentDeviceSlug) : null;
			if (!destination || !clipboard.mode) return "Open a folder on this device";
			return pasteBlocker(clipboard.mode, clipboard.items, destination);
		},
		[clipboard.items, clipboard.mode, currentDeviceSlug],
	);

	/** Copies or moves the held files into `folder`; a name already taken gets a new one. */
	const paste = useCallback(
		(folder: WingPath) => {
			const destination = localPath(folder, currentDeviceSlug);
			if (!destination) return;
			const source = useFileClipboard.getState().beginPaste(destination);
			if (!source) return;
			copy.mutate({
				sources: { paths: source.paths },
				destination: folder,
				overwrite: false,
				verify_checksum: false,
				preserve_timestamps: true,
				move_files: source.mode === "move",
				copy_method: "Auto",
				on_conflict: "AutoModifyName",
			});
		},
		[copy, currentDeviceSlug],
	);

	const localPathOf = useCallback(
		(file: File) => localPath(file.wing_path, currentDeviceSlug),
		[currentDeviceSlug],
	);

	return {
		element,
		localPathOf,
		clipboard,
		pasteBlockerIn,
		paste,
		hasActions,
		showActions: setSelected,
		openIfLocal,
		canCreateIn,
		canAddLocation,
		addAsLocation,
		newFolder: (parent: WingPath) => setPrompt({ kind: "newFolder", parent }),
	};
}
