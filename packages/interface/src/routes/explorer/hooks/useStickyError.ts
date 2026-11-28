import { useRef } from "react";

/** The fields of a TanStack query that decide which error to show. */
export interface QueryErrorState {
	error: Error | null;
	status: "pending" | "error" | "success";
}

/**
 * Returns the query's error, and keeps the last one while the query fetches again.
 *
 * TanStack Query clears `error` and goes back to `pending` when a query that has no
 * data starts a new fetch. A folder that could not be opened then flashed an empty
 * listing ("0 files") on every window focus or change event, until the fetch failed
 * again. The last error stays until a fetch succeeds or `key` changes.
 *
 * @param query - The active query, or `null` when no query backs the view.
 * @param key - Identifies what the query lists, such as its mode and path.
 */
export function useStickyError(
	query: QueryErrorState | null,
	key: string,
): Error | null {
	const last = useRef<{ key: string; error: Error } | null>(null);
	if (!query) {
		last.current = null;
		return null;
	}
	if (query.error) {
		last.current = { key, error: query.error };
		return query.error;
	}
	if (query.status === "success" || last.current?.key !== key) {
		last.current = null;
		return null;
	}
	return last.current.error;
}
