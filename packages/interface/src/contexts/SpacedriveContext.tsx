/**
 * Convenience re-exports from @wingdrive/ts-client
 *
 * You can import from here OR directly from @wingdrive/ts-client/hooks
 * Both work identically!
 */

// Re-export hooks from @wingdrive/ts-client (no longer duplicated!)
export {
	SpacedriveProvider,
	useSpacedriveClient,
	useClient,
	useCoreQuery,
	useLibraryQuery,
	useCoreMutation,
	useLibraryMutation,
	useNormalizedQuery,
} from "@wingdrive/ts-client/hooks";

// Export client type
export type { SpacedriveClient } from "@wingdrive/ts-client";

// Export commonly used types for convenience
export type {
	Location,
	LocationsListOutput,
	LibraryInfo,
} from "@wingdrive/ts-client";

// Export icon utilities
export { getDeviceIcon, getVolumeIcon } from "@wingdrive/ts-client";
export { WingDriveProvider, useWingDriveClient } from "@wingdrive/ts-client/hooks";
