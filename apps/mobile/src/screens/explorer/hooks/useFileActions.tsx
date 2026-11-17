import { useCallback, useState } from "react";
import { Alert } from "react-native";
import { useTranslation } from "react-i18next";
import { useQueryClient } from "@tanstack/react-query";
import type { File, WingPath } from "@wingdrive/ts-client";
import { fileDisplayName, isVirtualFile, useJobs } from "@wingdrive/ts-client";
import { WingMobileCore } from "wing-mobile-core";
import { ArrowSquareOut, PencilSimple, ShareNetwork, Trash } from "phosphor-react-native";
import { useLibraryAction, useNormalizedQuery } from "../../../client";
import { RenameDialog } from "../../../components/RenameDialog";
import { FileActionsSheet, type FileAction } from "../components/FileActionsSheet";
import { isInLocation, localPath } from "../fileLogic";

const ICON = "hsl(235, 10%, 55%)";

type Prompt = { kind: "rename"; file: File } | { kind: "newFolder"; parent: WingPath };

/**
 * Explorer file operations (MOB-007): open, share, rename and delete through a long press, and a
 * new folder from the header menu. Listings refetch when an operation's job ends.
 */
export function useFileActions() {
	const { t } = useTranslation("common");
	const queryClient = useQueryClient();
	const [selected, setSelected] = useState<File | null>(null);
	const [prompt, setPrompt] = useState<Prompt | null>(null);

	const { data: devices } = useNormalizedQuery({
		query: "devices.list",
		input: { include_offline: true, include_details: false, show_paired: true },
		resourceType: "device",
	});
	const currentDeviceSlug = devices?.find((device) => device.is_current)?.slug;

	const refetchListings = useCallback(() => {
		void queryClient.refetchQueries({ queryKey: ["query:files.directory_listing"], exact: false });
	}, [queryClient]);

	// Rename and delete run as jobs; the listing changes when they end.
	useJobs({ onJobCompleted: refetchListings, onJobFailed: refetchListings });

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
		onSuccess: (_output, input) =>
			Alert.alert(t("Location added"), input.name ?? undefined),
	});
	const createFolder = useLibraryAction("files.createFolder", {
		onError: showError,
		onSuccess: refetchListings,
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
			// Android has no trash for plain files, so a delete is permanent.
			Alert.alert(t("Delete permanently?"), `${fileDisplayName(file)}\n\n${t("This cannot be undone.")}`, [
				{ text: t("Cancel"), style: "cancel" },
				{
					text: t("Delete"),
					style: "destructive",
					onPress: () =>
						remove.mutate({
							targets: { paths: [file.wing_path] },
							permanent: true,
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
			actions.push(
				{
					label: "Rename",
					icon: <PencilSimple size={22} color={ICON} weight="bold" />,
					onPress: () => setPrompt({ kind: "rename", file }),
				},
				{
					label: "Delete",
					icon: <Trash size={22} color="#ef4444" weight="bold" />,
					onPress: () => confirmDelete(file),
					destructive: true,
				},
			);
			return actions;
		},
		[confirmDelete, currentDeviceSlug, openFile, showError],
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

	const localPathOf = useCallback(
		(file: File) => localPath(file.wing_path, currentDeviceSlug),
		[currentDeviceSlug],
	);

	return {
		element,
		localPathOf,
		hasActions,
		showActions: setSelected,
		openIfLocal,
		canCreateIn,
		canAddLocation,
		addAsLocation,
		newFolder: (parent: WingPath) => setPrompt({ kind: "newFolder", parent }),
	};
}
