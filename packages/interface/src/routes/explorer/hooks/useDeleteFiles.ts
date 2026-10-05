import { useCallback } from "react";
import { toast } from "@wingdrive/primitives";
import type { File } from "@wingdrive/ts-client";
import { useLibraryMutation } from "../../../contexts/WingDriveContext";
import { useDeleteConfirmationDialog } from "../../../components/modals/DeleteConfirmationModal";
import { useWaitForJob } from "../../../hooks/useWaitForJob";
import { useRefetchFileListings } from "../../../hooks/useRefetchFileListings";
import { summarizeFileOperation } from "../../../hooks/fileOperationOutcome";
import { reportFileOperation } from "../../../hooks/reportFileOperation";
import { isOperableFile } from "../fileCapabilities";

/**
 * Shared hook for delete file operations.
 * Used by both useExplorerKeyboard (DEL key) and useFileContextMenu.
 *
 * Confirmation happens in a styled dialog instead of the native `confirm()`
 * so destructive actions match the rest of the app's visual language.
 *
 * The mutation only queues a job, so the hook waits for the job to finish
 * before refreshing the listing. Without that wait the explorer refetched
 * while the files were still on disk and the rows never disappeared, which
 * made users delete the same file twice.
 */
export function useDeleteFiles() {
	const mutation = useLibraryMutation("files.delete");
	const openConfirmation = useDeleteConfirmationDialog();
	const waitForJob = useWaitForJob();
	const refetchListings = useRefetchFileListings();

	const deleteFiles = useCallback(
		async (files: File[], permanent: boolean) => {
			if (files.length === 0) return false;
			// A virtual entry carries the path of a whole location, volume or
			// device root; refusing here protects every caller, not just menus.
			if (!files.every(isOperableFile)) return false;
			if (mutation.isPending) return false;

			// Ask for confirmation in a dialog; resolves true if the user
			// confirms, false otherwise (dialog closed/cancelled)
			const confirmed = await new Promise<boolean>((resolve) => {
				openConfirmation({
					files,
					permanent,
					onConfirm: async () => {
						try {
							const { result } = await waitForJob(() =>
								mutation.mutateAsync({
									targets: { paths: files.map((f) => f.wing_path) },
									permanent,
									recursive: true,
								}),
							);
							refetchListings();

							const outcome = summarizeFileOperation("delete", result);
							reportFileOperation(outcome);
							resolve(outcome.status === "success");
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
		[mutation, openConfirmation, waitForJob, refetchListings],
	);

	return { deleteFiles, isPending: mutation.isPending };
}
