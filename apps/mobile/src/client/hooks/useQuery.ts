import {
	useQuery,
	useMutation,
	UseQueryOptions,
	UseMutationOptions,
} from "@tanstack/react-query";
import type {
	CoreAction,
	CoreQuery,
	LibraryAction,
	LibraryQuery,
} from "@wingdrive/ts-client/src/generated/types";
import { useSyncExternalStore } from "react";
import { useWingDriveClient } from "./useClient";

type Input<U, T> = Extract<U, { type: T }> extends { input: infer I } ? I : never;
type Output<U, T> = Extract<U, { type: T }> extends { output: infer O } ? O : never;

/**
 * Current library id that re-renders the caller when the library changes. Reading
 * `client.getCurrentLibraryId()` during render left screens on the previous library's data.
 */
export function useCurrentLibraryId(): string | null {
	const client = useWingDriveClient();
	return useSyncExternalStore(
		(onChange) => {
			client.on("library-changed", onChange);
			return () => client.off("library-changed", onChange);
		},
		() => client.getCurrentLibraryId(),
	);
}

/**
 * Hook for executing core-level queries (no library context).
 */
export function useCoreQuery<T extends CoreQuery["type"]>(
	method: T,
	input: Input<CoreQuery, T>,
	options?: Omit<UseQueryOptions<Output<CoreQuery, T>, Error>, "queryKey" | "queryFn">,
) {
	const client = useWingDriveClient();

	return useQuery<Output<CoreQuery, T>, Error>({
		queryKey: ["core", method, input],
		queryFn: () => client.coreQuery<Output<CoreQuery, T>>(method, input),
		...options,
	});
}

/**
 * Hook for executing library-level queries.
 * Automatically uses the current library context.
 */
export function useLibraryQuery<T extends LibraryQuery["type"]>(
	method: T,
	input: Input<LibraryQuery, T>,
	options?: Omit<UseQueryOptions<Output<LibraryQuery, T>, Error>, "queryKey" | "queryFn">,
) {
	const client = useWingDriveClient();
	const libraryId = useCurrentLibraryId();

	return useQuery<Output<LibraryQuery, T>, Error>({
		queryKey: ["library", libraryId, method, input],
		queryFn: () => client.libraryQuery<Output<LibraryQuery, T>>(method, input),
		enabled: !!libraryId && (options?.enabled ?? true),
		...options,
	});
}

/**
 * Hook for executing core-level actions (mutations).
 */
export function useCoreAction<T extends CoreAction["type"]>(
	method: T,
	options?: UseMutationOptions<Output<CoreAction, T>, Error, Input<CoreAction, T>>,
) {
	const client = useWingDriveClient();

	return useMutation<Output<CoreAction, T>, Error, Input<CoreAction, T>>({
		mutationFn: (input) => client.coreAction<Output<CoreAction, T>>(method, input),
		...options,
	});
}

/**
 * Hook for executing library-level actions (mutations).
 */
export function useLibraryAction<T extends LibraryAction["type"]>(
	method: T,
	options?: UseMutationOptions<Output<LibraryAction, T>, Error, Input<LibraryAction, T>>,
) {
	const client = useWingDriveClient();

	return useMutation<Output<LibraryAction, T>, Error, Input<LibraryAction, T>>({
		mutationFn: (input) => client.libraryAction<Output<LibraryAction, T>>(method, input),
		...options,
	});
}
