import {describe, expect, it} from 'bun:test';
import {TauriTransport} from '../transport';

/** In-memory stand-in for Tauri's per-window event bus and commands. */
function fakeTauri() {
	const listeners = new Map<string, Set<(event: {payload: unknown}) => void>>();
	const calls: {cmd: string; args: any}[] = [];
	let nextId = 1;
	let failSubscribe = 0;

	const listen = async (event: string, handler: (event: any) => void) => {
		if (!listeners.has(event)) listeners.set(event, new Set());
		listeners.get(event)!.add(handler);
		return () => listeners.get(event)?.delete(handler);
	};
	const invoke = async (cmd: string, args?: any) => {
		calls.push({cmd, args});
		if (cmd === 'subscribe_to_events') {
			if (failSubscribe > 0) {
				failSubscribe -= 1;
				throw new Error('daemon down');
			}
			return nextId++;
		}
		return null;
	};
	const emit = (event: string, payload: unknown) => {
		for (const handler of listeners.get(event) ?? []) handler({payload});
	};

	return {
		transport: new TauriTransport(invoke, listen),
		calls,
		emit,
		listeners,
		failNext: (n: number) => {
			failSubscribe = n;
		}
	};
}

const subscribeCalls = (calls: {cmd: string}[]) =>
	calls.filter((c) => c.cmd === 'subscribe_to_events');

describe('TauriTransport.subscribe', () => {
	it('receives only events sent to its own channel', async () => {
		const tauri = fakeTauri();
		const a: unknown[] = [];
		const b: unknown[] = [];
		await tauri.transport.subscribe((e) => a.push(e));
		await tauri.transport.subscribe((e) => b.push(e));

		const [first, second] = subscribeCalls(tauri.calls).map(
			(c: any) => c.args.channel
		);
		expect(first).not.toBe(second);

		tauri.emit(first, 'for-a');
		tauri.emit('core-event', 'broadcast');
		expect(a).toEqual(['for-a']);
		expect(b).toEqual([]);
	});

	it('reopens the stream after the daemon closes it', async () => {
		const tauri = fakeTauri();
		const received: unknown[] = [];
		await tauri.transport.subscribe((e) => received.push(e));
		const channel = (subscribeCalls(tauri.calls)[0] as any).args.channel;

		tauri.failNext(1);
		tauri.emit(`${channel}:closed`, 'event stream closed');
		await Bun.sleep(2000);

		// One failed retry while the daemon was down, then a working stream.
		expect(subscribeCalls(tauri.calls).length).toBe(3);
		tauri.emit(channel, 'after-restart');
		expect(received).toEqual(['after-restart']);
	});

	it('stops reopening once unsubscribed', async () => {
		const tauri = fakeTauri();
		const unsubscribe = await tauri.transport.subscribe(() => {});
		const channel = (subscribeCalls(tauri.calls)[0] as any).args.channel;

		await unsubscribe();
		tauri.emit(`${channel}:closed`, 'event stream closed');
		await Bun.sleep(700);

		expect(subscribeCalls(tauri.calls).length).toBe(1);
		expect(tauri.listeners.get(channel)?.size ?? 0).toBe(0);
		expect(tauri.calls.at(-1)?.cmd).toBe('unsubscribe_from_events');
	});

	it('cleans up its listeners when the first subscribe fails', async () => {
		const tauri = fakeTauri();
		tauri.failNext(1);
		await expect(tauri.transport.subscribe(() => {})).rejects.toThrow(
			'daemon down'
		);
		for (const set of tauri.listeners.values()) expect(set.size).toBe(0);
	});
});
