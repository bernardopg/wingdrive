import { useCallback } from "react";
import type { File } from "@wingdrive/ts-client";
import { useLibraryMutation } from "../../../contexts/WingDriveContext";
import { useDeleteConfirmationDialog } from "../../../components/modals/DeleteConfirmationModal";
import { useWaitForJob } from "../../../hooks/useWaitForJob";
import { useRefetchFileListings } from "../../../hooks/useRefetchFileListings";
import { useUndo } from "../../../hooks/useUndo";
import { summarizeFileOperation } from "../../../hooks/fileOperationOutcome";
import { reportFileOperation } from "../../../hooks/reportFileOperation";
import { useAdminRetry } from "../../../hooks/adminRetry";
import { isOperableFile } from "../fileCapabilities";
import { useTranslation } from "../../../i18n";

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
	const {t} = useTranslation("explorer");
	const mutation = useLibraryMutation("files.delete");
	const undo = useUndo();
	const openConfirmation = useDeleteConfirmationDialog();
	const waitForJob = useWaitForJob();
	const refetchListings = useRefetchFileListings();
	const adminRetry = useAdminRetry();

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
							const { receipt, result } = await waitForJob(() =>
								mutation.mutateAsync({
									targets: { paths: files.map((f) => f.wing_path) },
									permanent,
									recursive: true,
								}),
							);
							refetchListings();

							const outcome = summarizeFileOperation("delete", result);
							// Files the user cannot write can still go, as root, after a prompt.
							// Root has no access to the user's trash, so that removal is permanent.
							const localPaths = files.flatMap((f) =>
								f.is_local && "Physical" in f.wing_path ? [f.wing_path.Physical.path] : [],
							);
							const denied = outcome.details.some(adminRetry.canRetry);
							if (outcome.status !== "success" && denied && localPaths.length > 0) {
								const detail = outcome.details[0];
								adminRetry.report(
									detail ? `${outcome.message}. ${detail}` : outcome.message,
									{ type: "delete", paths: localPaths },
									detail,
								);
							} else {
								reportFileOperation(outcome);
							}
							// The delete job journals where each local item went in the
							// trash; permanent deletes record nothing and have no way back.
							const trashedLocal =
								!permanent && files.some((f) => undo.isLocalPath(f.wing_path));
							if (
								(outcome.status === "success" || outcome.status === "partial") &&
								trashedLocal
							) {
								undo.recordJournaled("delete", receipt.id);
							}
							resolve(outcome.status === "success");
						} catch (err) {
							console.error("Failed to delete:", err);
							const localPaths = files.flatMap((f) =>
								f.is_local && "Physical" in f.wing_path ? [f.wing_path.Physical.path] : [],
							);
							adminRetry.report(
								t("errors.failedToDelete", {error: err}),
								localPaths.length > 0 ? { type: "delete", paths: localPaths } : null,
								String(err),
							);
							resolve(false);
						}
					},
					onCancelled: () => resolve(false),
				});
			});

			return confirmed;
		},
		[mutation, openConfirmation, waitForJob, refetchListings, undo, adminRetry, t],
	);

	return { deleteFiles, isPending: mutation.isPending };
}
