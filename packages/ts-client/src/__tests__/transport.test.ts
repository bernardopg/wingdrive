import { afterEach, beforeEach, describe, expect, it, mock } from "bun:test";
import { HttpTransport } from "../transport";

class MockEventSource {
	static instances: MockEventSource[] = [];
	url: string;
	onmessage: ((event: { data: string }) => void) | null = null;
	onerror: ((event: unknown) => void) | null = null;
	close = mock(() => {});

	constructor(url: string) {
		this.url = url;
		MockEventSource.instances.push(this);
	}

	emit(data: unknown) {
		this.onmessage?.({ data: JSON.stringify(data) });
	}
}

describe("HttpTransport", () => {
	const originalFetch = globalThis.fetch;
	const originalEventSource = (globalThis as any).EventSource;

	beforeEach(() => {
		MockEventSource.instances = [];
		(globalThis as any).EventSource = MockEventSource;
	});

	afterEach(() => {
		globalThis.fetch = originalFetch;
		(globalThis as any).EventSource = originalEventSource;
	});

	it("posts RPC requests to /rpc and returns JSON", async () => {
		globalThis.fetch = mock(async (url: string, init: any) => {
			expect(url).toBe("http://localhost:8080/rpc");
			expect(init.method).toBe("POST");
			expect(init.headers).toEqual({ "Content-Type": "application/json" });
			expect(JSON.parse(init.body)).toEqual({ Ping: null });
			return new Response(JSON.stringify("Pong"), { status: 200 });
		}) as any;

		const transport = new HttpTransport("http://localhost:8080/");
		await expect(transport.sendRequest({ Ping: null })).resolves.toBe("Pong");
	});

	it("surfaces HTTP errors with response text", async () => {
		globalThis.fetch = mock(async () =>
			new Response("boom", { status: 500, statusText: "Internal Server Error" }),
		) as any;

		const transport = new HttpTransport("http://localhost:8080");
		await expect(transport.sendRequest({ Ping: null })).rejects.toThrow(
			"RPC 500 Internal Server Error: boom",
		);
	});

	it("multiplexes one EventSource across subscribers and filters envelopes", async () => {
		const transport = new HttpTransport("http://localhost:8080");
		const a: unknown[] = [];
		const b: unknown[] = [];

		const unsubA = await transport.subscribe((event) => a.push(event));
		const unsubB = await transport.subscribe((event) => b.push(event));

		expect(MockEventSource.instances).toHaveLength(1);
		expect(MockEventSource.instances[0].url).toBe("http://localhost:8080/events");

		MockEventSource.instances[0].emit({ Event: { JobStarted: { job_id: "1" } } });
		MockEventSource.instances[0].emit({ LogMessage: { message: "hi" } });
		MockEventSource.instances[0].emit({ Subscribed: true });
		MockEventSource.instances[0].onmessage?.({ data: "not-json" });

		expect(a).toEqual([{ JobStarted: { job_id: "1" } }, { message: "hi" }]);
		expect(b).toEqual(a);

		unsubA();
		expect(MockEventSource.instances[0].close).not.toHaveBeenCalled();
		unsubB();
		expect(MockEventSource.instances[0].close).toHaveBeenCalledTimes(1);
	});
});
