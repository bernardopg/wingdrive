import "./setup";
import { describe, expect, it, mock } from "bun:test";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { WingDriveClientContext } from "../useClient";
import { useSearchFiles } from "../useSearchFiles";

function createFakeClient(response: any = { files: [], total: 0 }) {
	const execute = mock(async (_method: string, _input: any) => response);
	const client: any = {
		getCurrentLibraryId: () => "lib-1",
		execute,
		on: () => {},
		off: () => {},
		subscribeFiltered: mock(async () => () => {}),
	};
	return { client, execute };
}

function wrapper(client: any) {
	const qc = new QueryClient({
		defaultOptions: { queries: { retry: false, gcTime: 0 } },
	});
	return ({ children }: { children: ReactNode }) => (
		<QueryClientProvider client={qc}>
			<WingDriveClientContext.Provider value={client}>
				{children}
			</WingDriveClientContext.Provider>
		</QueryClientProvider>
	);
}

describe("useSearchFiles", () => {
	it("does not run when query is shorter than 2 characters", async () => {
		const { client, execute } = createFakeClient();
		const { result } = renderHook(
			() => useSearchFiles({ query: "a", scope: "library" }),
			{ wrapper: wrapper(client) },
		);

		await waitFor(() => expect(result.current.isLoading).toBe(false));
		expect(execute).not.toHaveBeenCalled();
		expect(result.current.files).toEqual([]);
	});

	it("maps folder scope to path input and returns files", async () => {
		const path = { Physical: { device_slug: "mac", path: "/Desktop" } } as any;
		const { client, execute } = createFakeClient({
			files: [{ id: "1", name: "photo.jpg" }],
		});

		const { result } = renderHook(
			() =>
				useSearchFiles({
					query: "photo",
					scope: "folder",
					currentPath: path,
					sortBy: "Name",
					sortDirection: "Asc",
				}),
			{ wrapper: wrapper(client) },
		);

		await waitFor(() => expect(result.current.files).toHaveLength(1));
		expect(execute).toHaveBeenCalledTimes(1);
		const [method, input] = execute.mock.calls[0];
		expect(method).toBe("query:search.files");
		expect(input.query).toBe("photo");
		expect(input.scope).toEqual({ Path: { path } });
		expect(input.sort).toEqual({ field: "Name", direction: "Asc" });
	});

	it("maps location and library scopes", async () => {
		const location = createFakeClient({ files: [] });
		const { result: locationResult } = renderHook(
			() =>
				useSearchFiles({
					query: "test",
					scope: "location",
					locationId: "loc-1",
				}),
			{ wrapper: wrapper(location.client) },
		);
		await waitFor(() => expect(locationResult.current.isLoading).toBe(false));
		expect(location.execute.mock.calls[0][1].scope).toEqual({
			Location: { location_id: "loc-1" },
		});

		const library = createFakeClient({ files: [] });
		const { result: libraryResult } = renderHook(
			() => useSearchFiles({ query: "test", scope: "library" }),
			{ wrapper: wrapper(library.client) },
		);
		await waitFor(() => expect(libraryResult.current.isLoading).toBe(false));
		expect(library.execute.mock.calls[0][1].scope).toBe("Library");
	});
});
