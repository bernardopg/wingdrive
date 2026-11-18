import { useMemo } from "react";
import type { File, WingPath } from "@wingdrive/ts-client";
import { isLibraryView, libraryViewInput } from "../fileLogic";
import { useNormalizedQuery } from "@wingdrive/ts-client";
import { useVirtualListing } from "./useVirtualListing";

export type FileSource = "virtual" | "directory" | "search";

export interface ExplorerFilesResult {
	files: File[];
	isLoading: boolean;
	source: FileSource;
}

/**
 * Centralized hook for fetching files in the mobile explorer.
 *
 * Handles two file sources with priority:
 * 1. Virtual listings (devices/volumes/locations)
 * 2. Directory listings (normal file browsing)
 */
export function useExplorerFiles(
	params:
		| { type: "path"; path: string }
		| { type: "view"; view: string; id?: string }
		| undefined,
): ExplorerFilesResult {
	// Check for virtual listing first
	const libraryView = params?.type === "view" && isLibraryView(params.view) ? params.view : null;
	const { files: virtualFiles, isVirtualView, isLoading: virtualLoading } =
		useVirtualListing(libraryView ? undefined : params);

	// Parse path for directory listing
	const currentPath: WingPath | null = useMemo(() => {
		if (params?.type === "path") {
			try {
				return JSON.parse(params.path) as WingPath;
			} catch (e) {
				console.error("[useExplorerFiles] Failed to parse path:", e);
				return null;
			}
		}
		return null;
	}, [params]);

	// Directory query
	const directoryQuery = useNormalizedQuery({
		query: "files.directory_listing",
		input: currentPath
			? {
					path: currentPath,
					limit: null,
					include_hidden: false,
					sort_by: "name",
					folders_first: true,
				}
			: (null as any),
		resourceType: "file",
		enabled: !!currentPath && !isVirtualView,
		pathScope: currentPath ?? undefined,
	});

	const searchQuery = useNormalizedQuery({
		query: "search.files",
		input: libraryView ? libraryViewInput(libraryView) : (null as any),
		resourceType: "file",
		refetchOnResourceChange: libraryView === "favorites",
		enabled: !!libraryView,
	});

	// Determine source and files with priority: virtual > library view > directory
	const source: FileSource = isVirtualView ? "virtual" : libraryView ? "search" : "directory";

	const files = useMemo(() => {
		if (isVirtualView) {
			return virtualFiles || [];
		}
		if (libraryView) {
			return searchQuery.data?.files || [];
		}
		return (directoryQuery.data as { files?: File[] })?.files || [];
	}, [isVirtualView, virtualFiles, libraryView, searchQuery.data, directoryQuery.data]);

	const isLoading = isVirtualView
		? virtualLoading
		: libraryView
			? searchQuery.isLoading
			: directoryQuery.isLoading;

	return {
		files,
		isLoading,
		source,
	};
}
