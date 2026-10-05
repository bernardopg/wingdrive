import { useCallback } from "react";
import type { File, WingPath } from "@wingdrive/ts-client";
import { useLibraryMutation } from "../../../contexts/WingDriveContext";
import {
	combineFileOperations,
	summarizeFileOperation,
	type FileOperationOutcome,
} from "../../../hooks/fileOperationOutcome";
import { reportFileOperation } from "../../../hooks/reportFileOperation";
import { useWaitForJob } from "../../../hooks/useWaitForJob";
import { useRefetchFileListings } from "../../../hooks/useRefetchFileListings";
import { isOperableFile } from "../fileCapabilities";

/**
 * Shared hook for duplicating files in place.
 *
 * Duplication copies each file to a sibling path named "<name> copy<ext>"
 * using the single-source exact-path copy semantics of the backend copy job.
 * `AutoModifyName` conflict resolution keeps duplicates safe when the copy
 * name already exists (file copy.txt -> file copy (1).txt).
 *
 * Copy runs as a job, so the listing is only refreshed once the job reports
 * back; refetching earlier showed the folder without the new file. The jobs'
 * counts are merged so a duplicate that failed for some items is reported
 * instead of passing silently.
 */
export function useDuplicateFiles() {
	const mutation = useLibraryMutation("files.copy");
	const waitForJob = useWaitForJob();
	const refetchListings = useRefetchFileListings();

	const duplicateFiles = useCallback(
		async (files: File[]): Promise<FileOperationOutcome | null> => {
			if (files.length === 0 || !files.every(isOperableFile)) return null;
			if (mutation.isPending) return null;

			const outcomes = await Promise.all(
				files.map(async (file): Promise<FileOperationOutcome> => {
					const destination = buildDuplicateTarget(file);
					if (!destination) {
						return {
							status: "failed",
							message: `${file.name} has no local path to duplicate next to`,
							details: [],
							done: 0,
							failed: 1,
							skipped: 0,
						};
					}
					try {
						const { result } = await waitForJob(() =>
						mutation.mutateAsync({
							sources: { paths: [file.wing_path] },
							destination,
							overwrite: false,
							verify_checksum: false,
							preserve_timestamps: true,
							move_files: false,
							copy_method: "Auto",
							on_conflict: "AutoModifyName",
						}),
						);
						return summarizeFileOperation("duplicate", result);
					} catch (error) {
						return {
							status: "failed",
							message: `Could not duplicate ${file.name}: ${error}`,
							details: [],
							done: 0,
							failed: 1,
							skipped: 0,
						};
					}
				}),
			);

			refetchListings();
			const outcome = combineFileOperations("duplicate", outcomes);
			reportFileOperation(outcome);
			return outcome;
		},
		[mutation, waitForJob, refetchListings],
	);

	return { duplicateFiles, isPending: mutation.isPending };
}

function buildDuplicateTarget(file: File): WingPath | null {
	if (!("Physical" in file.wing_path)) return null;

	const { path, device_slug } = file.wing_path.Physical;
	const parent = getParentDir(path);
	const copyName = `${file.name} copy${file.extension ? `.${file.extension}` : ""}`;

	return {
		Physical: {
			device_slug,
			path: parent + copyName,
		},
	};
}

function getParentDir(path: string): string {
	const index = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
	return index >= 0 ? path.slice(0, index + 1) : "";
}