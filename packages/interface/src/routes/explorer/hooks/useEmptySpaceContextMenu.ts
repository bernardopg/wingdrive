import { FolderPlus, Copy } from "@phosphor-icons/react";
import { useContextMenu } from "../../../hooks/useContextMenu";
import { useLibraryMutation } from "../../../contexts/SpacedriveContext";
import { toast } from "@wingdrive/primitives";
import { useRefetchFileListings } from "../../../hooks/useRefetchFileListings";
import { useExplorer } from "../context";
import { useClipboard } from "../../../hooks/useClipboard";
import { useFileOperationDialog } from "../../../components/modals/FileOperationModal";

export function useEmptySpaceContextMenu() {
	const { currentPath } = useExplorer();
	const createFolder = useLibraryMutation("files.createFolder");
	const refetchListings = useRefetchFileListings();
	const clipboard = useClipboard();
	const openFileOperation = useFileOperationDialog();

	return useContextMenu({
		items: [
			{
				icon: FolderPlus,
				label: "New Folder",
				onClick: async () => {
					if (!currentPath) return;
					try {
						await createFolder.mutateAsync({
							parent: currentPath,
							name: "Untitled Folder",
							items: [],
						});
						// The mutation creates the folder without emitting a
						// listing event; without the manual refetch the new
						// folder only appeared after leaving and re-entering.
						refetchListings();
					} catch (err) {
						console.error("Failed to create folder:", err);
						toast.error(`Failed to create folder: ${err}`);
					}
				},
				condition: () => !!currentPath,
			},
			{
				icon: Copy,
				label: "Paste",
				onClick: () => {
					if (!clipboard.hasClipboard() || !currentPath) return;

					const operation =
						clipboard.operation === "cut" ? "move" : "copy";

					openFileOperation({
						operation,
						sources: clipboard.files,
						destination: currentPath,
						onComplete: () => {
							if (clipboard.operation === "cut") {
								clipboard.clearClipboard();
							}
						},
					});
				},
				keybindId: "explorer.paste",
				condition: () => clipboard.hasClipboard(),
			},
		],
	});
}
