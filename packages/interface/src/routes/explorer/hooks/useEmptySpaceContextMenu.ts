import { FolderPlus, Copy } from "@phosphor-icons/react";
import { useContextMenu } from "../../../hooks/useContextMenu";
import { useExplorer } from "../context";
import { useCreateFolder } from "./useCreateFolder";
import { useClipboard } from "../../../hooks/useClipboard";
import { useFileOperationDialog } from "../../../components/modals/FileOperationModal";

export function useEmptySpaceContextMenu() {
	const { currentPath } = useExplorer();
	const createFolder = useCreateFolder();
	const clipboard = useClipboard();
	const openFileOperation = useFileOperationDialog();

	return useContextMenu({
		items: [
			{
				icon: FolderPlus,
				label: "New Folder",
				onClick: createFolder,
				keybindId: "explorer.newFolder",
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
