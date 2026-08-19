import { useCallback } from "react";
import type { File, SdPath } from "@sd/ts-client";
import { useLibraryMutation } from "../../../contexts/SpacedriveContext";

/**
 * Shared hook for duplicating files in place.
 *
 * Duplication copies each file to a sibling path named "<name> copy<ext>"
 * using the single-source exact-path copy semantics of the backend copy job.
 * `AutoModifyName` conflict resolution keeps duplicates safe when the copy
 * name already exists (file copy.txt -> file copy (1).txt).
 */
export function useDuplicateFiles() {
	const mutation = useLibraryMutation("files.copy");

	const duplicateFiles = useCallback(
		async (files: File[]) => {
			if (files.length === 0) return;
			if (mutation.isPending) return;

			await Promise.all(
				files.map(async (file) => {
					const destination = buildDuplicateTarget(file);
					if (!destination) return;
					await mutation.mutateAsync({
						sources: { paths: [file.sd_path] },
						destination,
						overwrite: false,
						verify_checksum: false,
						preserve_timestamps: true,
						move_files: false,
						copy_method: "Auto",
						on_conflict: "AutoModifyName",
					});
				}),
			);
		},
		[mutation],
	);

	return { duplicateFiles, isPending: mutation.isPending };
}

function buildDuplicateTarget(file: File): SdPath | null {
	if (!("Physical" in file.sd_path)) return null;

	const { path, device_slug } = file.sd_path.Physical;
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