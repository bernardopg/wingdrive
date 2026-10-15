import {
	useNormalizedQuery as useSharedNormalizedQuery,
	type UseNormalizedQueryOptions,
} from "@wingdrive/ts-client/src/hooks/useNormalizedQuery";
import type { CoreQuery, LibraryQuery } from "@wingdrive/ts-client/src/generated/types";

type AnyQuery = CoreQuery | LibraryQuery;
type QueryInput<T> = Extract<AnyQuery, { type: T }> extends { input: infer I } ? I : never;
type QueryOutput<T> = Extract<AnyQuery, { type: T }> extends { output: infer O } ? O : never;

/**
 * The shared normalized query, typed by the query name.
 *
 * The shared hook takes its input and output types as free generics, so an
 * untyped call returns `any`; deriving both from the generated query unions
 * keeps mobile screens in step with the Rust types.
 */
export function useNormalizedQuery<
	T extends AnyQuery["type"],
	TSelected = QueryOutput<T>,
>(
	options: Omit<UseNormalizedQueryOptions<QueryInput<T>, QueryOutput<T>, TSelected>, "query"> & {
		query: T;
	},
) {
	return useSharedNormalizedQuery<QueryInput<T>, QueryOutput<T>, TSelected>(options);
}
