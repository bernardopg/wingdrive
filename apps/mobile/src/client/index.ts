// Core client
export { WingDriveClient } from "./WingDriveClient";
export { ReactNativeTransport } from "./transport";

// Provider and hooks
export { WingDriveProvider, useWingDriveClient } from "./hooks/useClient";
export {
	useCoreQuery,
	useLibraryQuery,
	useCoreAction,
	useLibraryAction,
} from "./hooks/useQuery";

// Re-export shared hooks from ts-client
export { useNormalizedQuery } from "@wingdrive/ts-client/src/hooks/useNormalizedQuery";
export { useSearchFiles } from "@wingdrive/ts-client";
