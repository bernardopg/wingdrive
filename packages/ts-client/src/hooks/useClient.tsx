import { createContext, useContext, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { WingDriveClient } from "../client";

// Export context so platforms can provide their own wrappers
export const WingDriveClientContext = createContext<WingDriveClient | null>(null);

// Create a singleton query client
export const queryClient = new QueryClient({
	defaultOptions: {
		queries: {
			staleTime: 30000, // 30 seconds
			gcTime: 300000, // 5 minutes
			retry: 1,
			refetchOnWindowFocus: true,
			refetchOnReconnect: true,
		},
	},
});

export interface WingDriveProviderProps {
	client: WingDriveClient;
	children: ReactNode;
}

/**
 * Provider for WingDriveClient + TanStack Query
 * Wrap your app with this to make the client available via hooks
 */
export function WingDriveProvider({ client, children }: WingDriveProviderProps) {
	return (
		<QueryClientProvider client={queryClient}>
			<WingDriveClientContext.Provider value={client}>
				{children}
			</WingDriveClientContext.Provider>
		</QueryClientProvider>
	);
}

/**
 * Hook to access the WingDrive client
 * Must be used within a WingDriveProvider
 */
export function useWingDriveClient(): WingDriveClient {
	const client = useContext(WingDriveClientContext);

	if (!client) {
		throw new Error("useWingDriveClient must be used within WingDriveProvider");
	}

	return client;
}

// Also export for direct use
export { useClient };
function useClient() {
	return useWingDriveClient();
}
