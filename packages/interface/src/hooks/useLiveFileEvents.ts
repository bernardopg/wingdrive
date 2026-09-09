import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useSpacedriveClient } from "@sd/ts-client";

/**
 * Query keys invalidated when the filesystem changes underneath the app.
 * Mirrors useRefetchFileListings so every view that renders file rows
 * converges on the same refresh set.
 */
const LISTING_QUERY_KEYS = [
	"query:files.directory_listing",
	"query:files.media_listing",
	"query:files.by_id",
	"query:files.recents",
];

/** Events that mean "the set of files may have changed". */
function isListingEvent(event: Record<string, unknown>): boolean {
	return (
		"FsRawChange" in event ||
		"EntryCreated" in event ||
		"EntryModified" in event ||
		"EntryDeleted" in event ||
		"EntryMoved" in event ||
		"FileOperationCompleted" in event ||
		"FilesModified" in event ||
		"FilesIndexed" in event ||
		"IndexingCompleted" in event
	);
}

/**
 * Keeps file listings in sync with real filesystem activity.
 *
 * The daemon already broadcasts watcher and indexer events, but nothing
 * consumed them: creating a file from the terminal, or an ephemeral folder
 * finishing its first index, never reached the explorer. This hook listens
 * for those events and invalidates the listing queries (debounced, since a
 * single bulk operation can emit hundreds of FsRawChange events).
 */
export function useLiveFileEvents() {
	const client = useSpacedriveClient();
	const queryClient = useQueryClient();

	useEffect(() => {
		if (!client) return;

		let unsubscribe: (() => void) | undefined;
		let cancelled = false;
		let timer: ReturnType<typeof setTimeout> | undefined;

		const scheduleInvalidate = () => {
			if (timer) return;
			timer = setTimeout(() => {
				timer = undefined;
				for (const key of LISTING_QUERY_KEYS) {
					queryClient.invalidateQueries({
						queryKey: [key],
						exact: false,
					});
				}
			}, 400);
		};

		client
			.subscribeFiltered(
				{
					event_types: [
						"FsRawChange",
						"EntryCreated",
						"EntryModified",
						"EntryDeleted",
						"EntryMoved",
						"FileOperationCompleted",
						"FilesModified",
						"FilesIndexed",
						"IndexingCompleted",
					],
				},
				(event) => {
					if (isListingEvent(event as Record<string, unknown>)) {
						scheduleInvalidate();
					}
				},
			)
			.then((unsub: () => void) => {
				if (cancelled) unsub();
				else unsubscribe = unsub;
			})
			.catch((error: unknown) => {
				if (cancelled) return;
				console.error("Failed to subscribe to file events", error);
			});

		return () => {
			cancelled = true;
			if (timer) clearTimeout(timer);
			unsubscribe?.();
		};
	}, [client, queryClient]);
}
