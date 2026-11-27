import { describe, expect, it, mock } from "bun:test";
import { SubscriptionManager } from "./subscriptionManager";
import type { Transport } from "./transport";

function createTransport() {
	const calls: any[] = [];
	const unsubscribes: Array<ReturnType<typeof mock>> = [];
	const callbacks: Array<(event: any) => void> = [];

	const transport: Transport = {
		sendRequest: mock(async () => ({})),
		subscribe: mock(async (callback: (event: any) => void) => {
			callbacks.push(callback);
			const unsub = mock(() => {});
			unsubscribes.push(unsub);
			calls.push({ callback });
			return unsub;
		}),
	};

	return { transport, calls, unsubscribes, callbacks };
}

describe("SubscriptionManager", () => {
	it("removes a failed subscription and permits a successful retry", async () => {
		const unsubscribe = mock(() => {});
		let attempts = 0;
		const transport: Transport = {
			sendRequest: mock(async () => ({})),
			subscribe: mock(async () => {
				if (++attempts === 1) throw new Error("subscription failed");
				return unsubscribe;
			}),
		};
		const manager = new SubscriptionManager(transport);
		await expect(manager.subscribe({ resource_type: "file" }, () => {})).rejects.toThrow("subscription failed");
		expect(manager.getStats().activeSubscriptions).toBe(0);
		const cleanup = await manager.subscribe({ resource_type: "file" }, () => {});
		expect(attempts).toBe(2);
		cleanup();
		expect(unsubscribe).toHaveBeenCalledTimes(1);
	});
	it("reuses a transport subscription for identical filters", async () => {
		const { transport } = createTransport();
		const manager = new SubscriptionManager(transport);
		const a = mock(() => {});
		const b = mock(() => {});

		const unsubA = await manager.subscribe(
			{ library_id: "lib-1", resource_type: "file" },
			a,
		);
		const unsubB = await manager.subscribe(
			{ library_id: "lib-1", resource_type: "file" },
			b,
		);

		expect(transport.subscribe).toHaveBeenCalledTimes(1);
		expect(manager.getStats().activeSubscriptions).toBe(1);
		expect(manager.getStats().subscriptions[0].refCount).toBe(2);

		unsubA();
		expect(manager.getStats().activeSubscriptions).toBe(1);
		unsubB();
		expect(manager.getStats().activeSubscriptions).toBe(0);
	});

	it("broadcasts events to all listeners for a shared subscription", async () => {
		const { transport, callbacks } = createTransport();
		const manager = new SubscriptionManager(transport);
		const a = mock(() => {});
		const b = mock(() => {});

		await manager.subscribe({ resource_type: "location" }, a);
		await manager.subscribe({ resource_type: "location" }, b);

		callbacks[0]({ ResourceChanged: { resource_type: "location" } });
		expect(a).toHaveBeenCalledTimes(1);
		expect(b).toHaveBeenCalledTimes(1);
	});

	it("creates separate subscriptions for different filters", async () => {
		const { transport } = createTransport();
		const manager = new SubscriptionManager(transport);

		await manager.subscribe({ resource_type: "file" }, () => {});
		await manager.subscribe({ resource_type: "location" }, () => {});

		expect(transport.subscribe).toHaveBeenCalledTimes(2);
		expect(manager.getStats().activeSubscriptions).toBe(2);
	});

	it("includes default event types when none are provided", async () => {
		const { transport } = createTransport();
		const manager = new SubscriptionManager(transport);

		await manager.subscribe({ resource_type: "file" }, () => {});

		expect(transport.subscribe).toHaveBeenCalledWith(expect.any(Function), {
			event_types: [
				"ResourceChanged",
				"ResourceChangedBatch",
				"ResourceDeleted",
				"Refresh",
			],
			filter: {
				resource_type: "file",
				path_scope: undefined,
				library_id: undefined,
				include_descendants: undefined,
			},
		});
	});

	it("passes custom event types and filter fields through", async () => {
		const { transport } = createTransport();
		const manager = new SubscriptionManager(transport);
		const pathScope = {
			Physical: { device_slug: "mac", path: "/Desktop" },
		} as any;

		await manager.subscribe(
			{
				library_id: "lib-1",
				resource_type: "file",
				path_scope: pathScope,
				include_descendants: true,
				event_types: ["ResourceChangedBatch"],
			},
			() => {},
		);

		expect(transport.subscribe).toHaveBeenCalledWith(expect.any(Function), {
			event_types: ["ResourceChangedBatch"],
			filter: {
				resource_type: "file",
				path_scope: pathScope,
				library_id: "lib-1",
				include_descendants: true,
			},
		});
	});

	it("shares one in-flight subscription across concurrent callers", async () => {
		const unsub = mock(() => {});
		let resolveSubscribe: ((value: () => void) => void) | undefined;
		const transport: Transport = {
			sendRequest: mock(async () => ({})),
			subscribe: mock(
				() =>
					new Promise<() => void>((resolve) => {
						resolveSubscribe = resolve;
					}),
			),
		};
		const manager = new SubscriptionManager(transport);
		const a = mock(() => {});
		const b = mock(() => {});

		const p1 = manager.subscribe({ resource_type: "file" }, a);
		const p2 = manager.subscribe({ resource_type: "file" }, b);
		expect(transport.subscribe).toHaveBeenCalledTimes(1);

		resolveSubscribe?.(unsub);
		const [unsubA, unsubB] = await Promise.all([p1, p2]);

		expect(manager.getStats().subscriptions[0].refCount).toBe(2);
		unsubA();
		unsubB();
		expect(unsub).toHaveBeenCalledTimes(1);
	});

	it("destroy unsubscribes all active subscriptions", async () => {
		const { transport, unsubscribes } = createTransport();
		const manager = new SubscriptionManager(transport);

		await manager.subscribe({ resource_type: "file" }, () => {});
		await manager.subscribe({ resource_type: "location" }, () => {});
		manager.destroy();

		expect(unsubscribes[0]).toHaveBeenCalledTimes(1);
		expect(unsubscribes[1]).toHaveBeenCalledTimes(1);
		expect(manager.getStats().activeSubscriptions).toBe(0);
	});
});
