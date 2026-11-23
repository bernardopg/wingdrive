import {describe, expect, it} from 'bun:test';

import {WingDriveClient, describeDaemonError} from '../client';
import type {Event} from '../generated/types';
import type {EventFilter, SubscriptionOptions, Transport} from '../transport';

class MockTransport implements Transport {
	requests: any[] = [];
	responses: any[] = [];
	subscribeCalls: {callback: (event: Event) => void; options?: SubscriptionOptions}[] = [];

	async sendRequest(request: any): Promise<any> {
		this.requests.push(request);
		if (this.responses.length === 0) throw new Error('no mock response');
		return this.responses.shift();
	}

	async subscribe(
		callback: (event: Event) => void,
		options?: SubscriptionOptions,
	): Promise<() => void> {
		this.subscribeCalls.push({callback, options});
		return () => {};
	}
}

describe('describeDaemonError', () => {
	it('returns strings directly', () => {
		expect(describeDaemonError('plain failure')).toBe('plain failure');
	});

	it('returns the message from single-key error objects', () => {
		expect(describeDaemonError({OperationFailed: 'Invalid input: bad path'})).toBe(
			'Invalid input: bad path',
		);
	});

	it('returns the variant name when detail is null', () => {
		expect(describeDaemonError({NotFound: null})).toBe('NotFound');
	});

	it('falls back to JSON for unknown shapes', () => {
		expect(describeDaemonError({a: 1, b: 2})).toBe('{"a":1,"b":2}');
		expect(describeDaemonError(42)).toBe('42');
	});
});

describe('WingDriveClient', () => {
	it('tracks library context and emits changes', async () => {
		const transport = new MockTransport();
		const client = new WingDriveClient(transport);
		const changes: string[] = [];
		client.on('library-changed', (id: string) => changes.push(id));

		expect(client.getCurrentLibraryId()).toBeNull();
		client.setCurrentLibrary('lib-1');
		expect(client.getCurrentLibraryId()).toBe('lib-1');
		expect(changes).toEqual(['lib-1']);

		client.setCurrentLibrary('lib-2', false);
		expect(client.getCurrentLibraryId()).toBe('lib-2');
		expect(changes).toEqual(['lib-1']);

		client.clearCurrentLibrary();
		expect(client.getCurrentLibraryId()).toBeNull();
	});

	it('requires a library before library-scoped execution context is used', () => {
		const client = new WingDriveClient(new MockTransport());
		expect(() => client.requireCurrentLibrary()).toThrow(
			'This operation requires an active library',
		);
		client.setCurrentLibrary('lib-1', false);
		expect(client.requireCurrentLibrary()).toBe('lib-1');
	});

	it('executes queries and actions with library context', async () => {
		const transport = new MockTransport();
		transport.responses.push({json: {items: []}}, {JsonOk: {ok: true}});
		const client = new WingDriveClient(transport);
		client.setCurrentLibrary('lib-9', false);

		await expect(client.execute('query:files.list', {path: '/'})).resolves.toEqual({
			items: [],
		});
		await expect(client.execute('action:files.delete', {id: '1'})).resolves.toEqual({
			ok: true,
		});

		expect(transport.requests[0]).toEqual({
			Query: {
				method: 'query:files.list',
				library_id: 'lib-9',
				payload: {path: '/'},
			},
		});
		expect(transport.requests[1]).toEqual({
			Action: {
				method: 'action:files.delete',
				library_id: 'lib-9',
				payload: {id: '1'},
			},
		});
	});

	it('surfaces daemon errors and rejects invalid wire methods', async () => {
		const transport = new MockTransport();
		transport.responses.push({error: {OperationFailed: 'bad input'}});
		const client = new WingDriveClient(transport);

		await expect(client.execute('query:test', {})).rejects.toThrow('bad input');
		await expect(client.execute('nope:test', {})).rejects.toThrow('Invalid wire method');
	});

	it('switches libraries only when the target exists', async () => {
		const transport = new MockTransport();
		transport.responses.push({json: [{id: 'lib-1'}, {id: 'lib-2'}]});
		const client = new WingDriveClient(transport);

		await client.switchToLibrary('lib-2');
		expect(client.getCurrentLibraryId()).toBe('lib-2');

		transport.responses.push({json: [{id: 'lib-1'}]});
		await expect(client.switchToLibrary('missing')).rejects.toThrow(
			"Library with ID 'missing' not found",
		);
	});

	it('returns current library info', async () => {
		const transport = new MockTransport();
		const client = new WingDriveClient(transport);
		expect(await client.getCurrentLibraryInfo()).toBeNull();

		client.setCurrentLibrary('lib-2', false);
		transport.responses.push({json: [{id: 'lib-1'}, {id: 'lib-2', name: 'Main'}]});
		await expect(client.getCurrentLibraryInfo()).resolves.toEqual({
			id: 'lib-2',
			name: 'Main',
		});
	});

	it('delegates filtered subscriptions to the subscription manager', async () => {
		const transport = new MockTransport();
		const client = new WingDriveClient(transport);
		const received: Event[] = [];

		const unsub = await client.subscribeFiltered(
			{library_id: 'lib-1', resource_type: 'file'},
			(event) => received.push(event),
		);
		expect(transport.subscribeCalls).toHaveLength(1);

		transport.subscribeCalls[0].callback({CoreStarted: null} as Event);
		expect(received).toHaveLength(1);
		expect(client.getSubscriptionStats().activeSubscriptions).toBe(1);

		unsub();
		expect(client.getSubscriptionStats().activeSubscriptions).toBe(0);
	});

	it('pings successfully and reports unexpected responses', async () => {
		const transport = new MockTransport();
		transport.responses.push('Pong', 'Nope');
		const client = new WingDriveClient(transport);

		await expect(client.ping()).resolves.toBeUndefined();
		await expect(client.ping()).rejects.toThrow('Unexpected ping response');
	});
});
