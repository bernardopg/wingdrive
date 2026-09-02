import { useCallback } from "react";
import { toast } from "@wingdrive/primitives";
import type { File } from "@sd/ts-client";
import { useLibraryMutation } from "../../../contexts/SpacedriveContext";
import { useDeleteConfirmationDialog } from "../../../components/modals/DeleteConfirmationModal";
import { useWaitForJob } from "../../../hooks/useWaitForJob";
import { useRefetchFileListings } from "../../../hooks/useRefetchFileListings";

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
							const receipt = await mutation.mutateAsync({
								targets: { paths: files.map((f) => f.sd_path) },
								permanent,
								recursive: true,
							});

							const result = await waitForJob(receipt.id);
							refetchListings();

							if (result.status === "failed") {
								toast.error(`Failed to delete: ${result.error}`);
								resolve(false);
								return;
							}

							if (
								result.status === "completed" &&
								result.output.type === "FileDelete" &&
								result.output.data.failed_count > 0
							) {
								const { deleted_count, failed_count } =
									result.output.data;
								toast.error(
									deleted_count > 0
										? `Deleted ${deleted_count}, failed ${failed_count}`
										: `Failed to delete ${failed_count} item${failed_count > 1 ? "s" : ""}`,
								);
								resolve(false);
								return;
							}

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
		[mutation, openConfirmation, waitForJob, refetchListings],
	);

	return { deleteFiles, isPending: mutation.isPending };
}
