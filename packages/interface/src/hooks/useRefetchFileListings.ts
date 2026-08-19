import { useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";

/**
 * Refetches every query that renders file rows.
 *
 * Delete and copy jobs mutate the filesystem without touching the database, so
 * no resource event reaches the explorer and stale rows stay on screen. Until
 * those jobs emit proper events, the views have to be refreshed by hand once
 * the job finishes.
 */
export function useRefetchFileListings() {
	const queryClient = useQueryClient();

	return useCallback(() => {
		for (const key of [
			"query:files.directory_listing",
			"query:files.media_listing",
			"query:files.by_id",
			"query:files.recents",
		]) {
			queryClient.refetchQueries({
				queryKey: [key],
				exact: false,
				type: "all",
			});
		}
	}, [queryClient]);
}
