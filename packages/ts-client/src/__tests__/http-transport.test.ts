import {afterEach, describe, expect, it} from 'bun:test';

import {HttpTransport} from '../transport';

class MockEventSource {
	static instances: MockEventSource[] = [];
	url: string;
	onmessage: ((e: {data: string}) => void) | null = null;
	onerror: ((e: unknown) => void) | null = null;
	closed = false;

	constructor(url: string) {
		this.url = url;
		MockEventSource.instances.push(this);
	}

	emit(data: unknown) {
		this.onmessage?.({data: JSON.stringify(data)});
	}

	emitRaw(data: string) {
		this.onmessage?.({data});
	}

	close() {
		this.closed = true;
	}
}

const originalFetch = globalThis.fetch;
const originalEventSource = (globalThis as any).EventSource;

afterEach(() => {
	globalThis.fetch = originalFetch;
	(globalThis as any).EventSource = originalEventSource;
	MockEventSource.instances = [];
});

describe('HttpTransport', () => {
	it('posts RPC requests to /rpc and returns JSON', async () => {
		const calls: any[] = [];
		globalThis.fetch = async (url: any, init?: any) => {
			calls.push({url, init});
			return new Response(JSON.stringify({json: {ok: true}}), {status: 200});
		};

		const transport = new HttpTransport('http://localhost:8080/');
		const result = await transport.sendRequest({Query: {method: 'q'}});

		expect(result).toEqual({json: {ok: true}});
		expect(calls[0].url).toBe('http://localhost:8080/rpc');
		expect(calls[0].init.method).toBe('POST');
		expect(calls[0].init.headers).toEqual({'Content-Type': 'application/json'});
	});

	it('throws a descriptive error for non-ok responses', async () => {
		globalThis.fetch = async () =>
			new Response('bad', {status: 500, statusText: 'Internal Server Error'});

		const transport = new HttpTransport('http://localhost:8080');
		await expect(transport.sendRequest({})).rejects.toThrow(
			'RPC 500 Internal Server Error: bad',
		);
	});

	it('multiplexes subscribers over a single EventSource', async () => {
		(globalThis as any).EventSource = MockEventSource;
		const transport = new HttpTransport('http://localhost:8080');
		const a: any[] = [];
		const b: any[] = [];

		const unsubA = await transport.subscribe((e) => a.push(e));
		const unsubB = await transport.subscribe((e) => b.push(e));
		expect(MockEventSource.instances).toHaveLength(1);
		expect(MockEventSource.instances[0].url).toBe('http://localhost:8080/events');

		MockEventSource.instances[0].emit({Event: {CoreStarted: null}});
		MockEventSource.instances[0].emit({LogMessage: {line: 'hi'}});
		MockEventSource.instances[0].emit({Unknown: true});
		MockEventSource.instances[0].emitRaw('not json');

		expect(a).toEqual([{CoreStarted: null}, {line: 'hi'}]);
		expect(b).toEqual([{CoreStarted: null}, {line: 'hi'}]);

		unsubA();
		expect(MockEventSource.instances[0].closed).toBe(false);
		unsubB();
		expect(MockEventSource.instances[0].closed).toBe(true);
	});
});
