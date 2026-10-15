import type { ContentKind, File } from "./generated/types";

/**
 * Get the content kind for a file, preferring content_identity.kind if available,
 * falling back to content_kind (identified by extension during ephemeral indexing).
 */
export function getContentKind(file: File | null | undefined): ContentKind {
	return file?.content_identity?.kind ?? file?.content_kind ?? "unknown";
}

/**
 * Get the appropriate kind string for icon resolution.
 * This transforms the content kind into a capitalized string suitable for icon lookup.
 */
export function getFileKindForIcon(file: File | null | undefined): string {
	const contentKind = getContentKind(file);
	const fileKind =
		contentKind && contentKind !== "unknown"
			? contentKind
			: file?.kind === "File"
				? file.extension || "File"
				: file?.kind || "File";
	return fileKind.charAt(0).toUpperCase() + fileKind.slice(1);
}

/** Name as shown to the user: `name` excludes the extension, so add it back. */
export function fileDisplayName(file: Pick<File, "name" | "extension">): string {
	return file.extension ? `${file.name}.${file.extension}` : file.name;
}

/**
 * Returns the content size safe to show to people.
 *
 * A directory's filesystem metadata length is not its content total, so it is
 * intentionally unavailable until the daemon has aggregated or measured it.
 */
export function fileDisplaySize(
	file: Pick<File, "kind" | "size" | "directory_size">,
): number | null {
	return file.kind === "Directory" ? file.directory_size : file.size;
}

/**
 * Applies on-demand directory totals (from `files.directory_sizes`) to a listing.
 *
 * A measured total replaces any stored aggregate, since it reflects the disk now.
 * Keys are the physical paths the daemon returned for each child directory.
 */
export function withDirectorySizes<T extends Pick<File, "kind" | "wing_path" | "directory_size">>(
	files: T[],
	sizes: Partial<Record<string, number>> | undefined,
): T[] {
	if (!sizes) return files;

	return files.map((file) => {
		if (file.kind !== "Directory" || !("Physical" in file.wing_path)) {
			return file;
		}

		const directorySize = sizes[file.wing_path.Physical.path];
		return directorySize === undefined || directorySize === file.directory_size
			? file
			: { ...file, directory_size: directorySize };
	});
}
