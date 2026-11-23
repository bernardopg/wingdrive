import {describe, expect, it} from 'bun:test';

import {TcpSocketTransport} from '../transport';

describe('TcpSocketTransport', () => {
	it('sends a JSON request and resolves a JSON response line', async () => {
		const requests: string[] = [];
		const server = Bun.listen({
			hostname: '127.0.0.1',
			port: 0,
			socket: {
				data(socket, data) {
					requests.push(new TextDecoder().decode(data));
					socket.write(JSON.stringify({json: {ok: true}}) + '\n');
				},
			},
		});

		try {
			const transport = new TcpSocketTransport(`127.0.0.1:${server.port}`);
			const response = await transport.sendRequest({Query: {method: 'q'}});
			expect(response).toEqual({json: {ok: true}});
			expect(JSON.parse(requests[0])).toEqual({Query: {method: 'q'}});
		} finally {
			server.stop(true);
		}
	});

	it('forwards subscribed events and ignores ack messages', async () => {
		const server = Bun.listen({
			hostname: '127.0.0.1',
			port: 0,
			socket: {
				data(socket) {
					socket.write(JSON.stringify('Subscribed') + '\n');
					socket.write(JSON.stringify({Event: {CoreStarted: null}}) + '\n');
					socket.write(JSON.stringify({LogMessage: {line: 'hi'}}) + '\n');
				},
			},
		});

		try {
			const transport = new TcpSocketTransport(`127.0.0.1:${server.port}`);
			const received: any[] = [];
			const unsub = await transport.subscribe((event) => received.push(event));
			await Bun.sleep(50);
			expect(received).toEqual([{CoreStarted: null}, {line: 'hi'}]);
			unsub();
		} finally {
			server.stop(true);
		}
	});
});
