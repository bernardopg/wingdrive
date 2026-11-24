import "./setup";
import { describe, expect, it, mock } from "bun:test";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { WingDriveClientContext } from "../useClient";
import { useJobs } from "../useJobs";

function createFakeClient(initialJobs: any[] = []) {
	let jobs = initialJobs.map((j) => ({ ...j }));
	const listeners: Array<(event: any) => void> = [];
	const execute = mock(async (method: string, input: any) => {
		if (method === "query:jobs.list") return { jobs: jobs.map((j) => ({ ...j })) };
		if (method === "action:jobs.pause.input") return { success: true, job_id: input.job_id };
		if (method === "action:jobs.resume.input") return { success: true, job_id: input.job_id };
		if (method === "action:jobs.cancel.input") return { success: true, job_id: input.job_id };
		throw new Error(`Unexpected method: ${method}`);
	});

	const client: any = {
		getCurrentLibraryId: () => "lib-1",
		execute,
		on: () => {},
		off: () => {},
		subscribeFiltered: mock(async (_filter: any, cb: (event: any) => void) => {
			listeners.push(cb);
			return () => {
				const idx = listeners.indexOf(cb);
				if (idx >= 0) listeners.splice(idx, 1);
			};
		}),
	};

	return {
		client,
		execute,
		emit: (event: any) => listeners.forEach((l) => l(event)),
		setJobs: (next: any[]) => {
			jobs = next.map((j) => ({ ...j }));
		},
	};
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

describe("useJobs", () => {
	it("loads jobs and exposes active counts", async () => {
		const { client } = createFakeClient([
			{ id: "1", status: "running", progress: 0.2 },
			{ id: "2", status: "paused", progress: 0.5 },
			{ id: "3", status: "completed", progress: 1 },
		]);

		const { result } = renderHook(() => useJobs(), { wrapper: wrapper(client) });
		await waitFor(() => expect(result.current.jobs).toHaveLength(3));

		expect(result.current.activeJobCount).toBe(2);
		expect(result.current.hasRunningJobs).toBe(true);
	});

	it("sends pause, resume and cancel mutations", async () => {
		const { client, execute } = createFakeClient([]);
		const { result } = renderHook(() => useJobs(), { wrapper: wrapper(client) });
		await waitFor(() => expect(result.current.isLoading).toBe(false));

		await act(async () => {
			await result.current.pause("job-1");
			await result.current.resume("job-1");
			await result.current.cancel("job-1");
		});

		expect(execute).toHaveBeenCalledWith("action:jobs.pause.input", { job_id: "job-1" });
		expect(execute).toHaveBeenCalledWith("action:jobs.resume.input", { job_id: "job-1" });
		expect(execute).toHaveBeenCalledWith("action:jobs.cancel.input", { job_id: "job-1" });
	});

	it("updates jobs from progress and completion events", async () => {
		const onJobCompleted = mock(() => {});
		const fake = createFakeClient([
			{ id: "job-1", status: "running", progress: 0 },
		]);

		const { result } = renderHook(() => useJobs({ onJobCompleted }), {
			wrapper: wrapper(fake.client),
		});
		await waitFor(() => expect(result.current.jobs).toHaveLength(1));
		await waitFor(() => expect(fake.client.subscribeFiltered).toHaveBeenCalledTimes(1));

		act(() => {
			fake.emit({
				JobProgress: {
					job_id: "job-1",
					progress: 0.5,
					generic_progress: {
						phase: "copying",
						current_path: "/a",
						message: "Copying",
						performance: { rate: 100 },
					},
				},
			});
		});

		await waitFor(() => expect(result.current.jobs[0].progress).toBe(0.5));
		expect(result.current.jobs[0].current_phase).toBe("copying");
		expect(result.current.getSpeedHistory("job-1")).toHaveLength(1);

		fake.setJobs([{ id: "job-1", status: "completed", progress: 1 }]);
		act(() => {
			fake.emit({
				JobCompleted: { job_id: "job-1", job_type: "copy", output: { ok: true } },
			});
		});

		await waitFor(() => expect(onJobCompleted).toHaveBeenCalledWith("job-1", "copy", { ok: true }));
		expect(result.current.getSpeedHistory("job-1")).toEqual([]);
	});
});
