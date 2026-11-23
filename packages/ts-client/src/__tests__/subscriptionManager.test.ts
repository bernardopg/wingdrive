import {describe, expect, it} from 'bun:test';

import type {Event} from '../generated/types';
import {SubscriptionManager} from '../subscriptionManager';
import type {EventFilter, SubscriptionOptions, Transport} from '../transport';

class MockTransport implements Transport {
	public subscribeCalls: SubscriptionOptions[] = [];
	public unsubscribed: string[] = [];
	private handlers = new Map<string, (event: Event) => void>();
	private nextId = 0;
	public failNextSubscribe = false;

	async sendRequest(): Promise<any> {
		return null;
	}

	async subscribe(
		callback: (event: Event) => void,
		options?: SubscriptionOptions,
	): Promise<() => void> {
		if (this.failNextSubscribe) {
			this.failNextSubscribe = false;
			throw new Error('subscribe failed');
		}
		const id = String(++this.nextId);
		this.handlers.set(id, callback);
		this.subscribeCalls.push(options ?? {});
		return () => {
			this.handlers.delete(id);
			this.unsubscribed.push(id);
		};
	}

	emit(id: string, event: Event) {
		this.handlers.get(id)?.(event);
	}

	emitAll(event: Event) {
		for (const handler of this.handlers.values()) handler(event);
	}
}

describe('SubscriptionManager', () => {
	it('shares a transport subscription for identical filters', async () => {
		const transport = new MockTransport();
		const manager = new SubscriptionManager(transport);
		const receivedA: Event[] = [];
		const receivedB: Event[] = [];

		const unsubA = await manager.subscribe(
			{library_id: 'lib1', resource_type: 'file'},
			(e) => receivedA.push(e),
		);
		const unsubB = await manager.subscribe(
			{library_id: 'lib1', resource_type: 'file'},
			(e) => receivedB.push(e),
		);

		expect(transport.subscribeCalls).toHaveLength(1);
		expect(manager.getStats().activeSubscriptions).toBe(1);
		expect(manager.getStats().subscriptions[0].refCount).toBe(2);

		transport.emitAll({CoreStarted: null} as Event);
		expect(receivedA).toHaveLength(1);
		expect(receivedB).toHaveLength(1);

		unsubA();
		expect(manager.getStats().activeSubscriptions).toBe(1);
		expect(transport.unsubscribed).toHaveLength(0);

		unsubB();
		expect(manager.getStats().activeSubscriptions).toBe(0);
		expect(transport.unsubscribed).toEqual(['1']);
	});

	it('creates separate subscriptions for different filters', async () => {
		const transport = new MockTransport();
		const manager = new SubscriptionManager(transport);

		await manager.subscribe({library_id: 'lib1'}, () => {});
		await manager.subscribe({library_id: 'lib2'}, () => {});

		expect(transport.subscribeCalls).toHaveLength(2);
		expect(manager.getStats().activeSubscriptions).toBe(2);
	});

	it('reuses an in-flight subscription for concurrent identical requests', async () => {
		let resolveSubscribe!: (unsub: () => void) => void;
		const transport: Transport = {
			sendRequest: async () => null,
			subscribe: () =>
				new Promise((resolve) => {
					resolveSubscribe = resolve as (unsub: () => void) => void;
				}),
		};
		const manager = new SubscriptionManager(transport);

		const p1 = manager.subscribe({resource_type: 'file'}, () => {});
		const p2 = manager.subscribe({resource_type: 'file'}, () => {});
		resolveSubscribe(() => {});

		const [unsub1, unsub2] = await Promise.all([p1, p2]);
		expect(manager.getStats().subscriptions[0].refCount).toBe(2);

		unsub1();
		unsub2();
		expect(manager.getStats().activeSubscriptions).toBe(0);
	});

	it('cleans up after a failed subscription attempt', async () => {
		const transport = new MockTransport();
		transport.failNextSubscribe = true;
		const manager = new SubscriptionManager(transport);

		await expect(
			manager.subscribe({resource_type: 'file'}, () => {}),
		).rejects.toThrow('subscribe failed');
		expect(manager.getStats().activeSubscriptions).toBe(0);

		const unsub = await manager.subscribe({resource_type: 'file'}, () => {});
		expect(transport.subscribeCalls).toHaveLength(1);
		unsub();
	});

	it('destroy unsubscribes all active subscriptions', async () => {
		const transport = new MockTransport();
		const manager = new SubscriptionManager(transport);

		await manager.subscribe({library_id: 'a'}, () => {});
		await manager.subscribe({library_id: 'b'}, () => {});
		expect(manager.getStats().activeSubscriptions).toBe(2);

		manager.destroy();
		expect(manager.getStats().activeSubscriptions).toBe(0);
		expect(transport.unsubscribed).toEqual(['1', '2']);
	});

	it('applies default event types when none are provided', async () => {
		const transport = new MockTransport();
		const manager = new SubscriptionManager(transport);

		const unsub = await manager.subscribe({resource_type: 'file'}, () => {});
		expect(transport.subscribeCalls[0].event_types).toEqual([
			'ResourceChanged',
			'ResourceChangedBatch',
			'ResourceDeleted',
			'Refresh',
		]);
		unsub();
	});
});
