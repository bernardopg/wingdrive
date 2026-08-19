import { useCallback } from "react";
import { toast } from "@spacedrive/primitives";
import type { File } from "@sd/ts-client";
import { useLibraryMutation } from "../../../contexts/SpacedriveContext";
import { useDeleteConfirmationDialog } from "../../../components/modals/DeleteConfirmationModal";

/**
 * Shared hook for delete file operations.
 * Used by both useExplorerKeyboard (DEL key) and useFileContextMenu.
 *
 * Confirmation happens in a styled dialog instead of the native `confirm()`
 * so destructive actions match the rest of the app's visual language.
 */
export function useDeleteFiles() {
	const mutation = useLibraryMutation("files.delete");
	const openConfirmation = useDeleteConfirmationDialog();

	const deleteFiles = useCallback(
		async (files: File[], permanent: boolean) => {
			if (files.length === 0) return false;
			if (files.some((f) => !f.sd_path)) return false;
			if (mutation.isPending) return false;

			// Ask for confirmation in a dialog; resolves true if the user
			// confirms, false otherwise (dialog closed/cancelled)
			const confirmed = await new Promise<boolean>((resolve) => {
				openConfirmation({
					files,
					permanent,
					onConfirm: async () => {
						try {
							await mutation.mutateAsync({
								targets: { paths: files.map((f) => f.sd_path) },
								permanent,
								recursive: true,
							});
							resolve(true);
						} catch (err) {
							console.error("Failed to delete:", err);
							toast.error(`Failed to delete: ${err}`);
							resolve(false);
						}
					},
					onCancelled: () => resolve(false),
				});
			});

			return confirmed;
		},
		[mutation, openConfirmation],
	);

	return { deleteFiles, isPending: mutation.isPending };
}