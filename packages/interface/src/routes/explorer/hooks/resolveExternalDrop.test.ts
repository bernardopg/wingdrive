import type {SdPath} from '@sd/ts-client';
import {describe, expect, it} from 'bun:test';
import {resolveExternalDrop} from './resolveExternalDrop';

const physicalFolder = (path: string): SdPath => ({
	Physical: {device_slug: 'local', path}
});

const remoteDestination = (path: string): SdPath => ({
	Physical: {device_slug: 'other-device', path}
});

const cloudDestination = (path: string): SdPath => ({
	Cloud: {
		service: 'S3',
		identifier: 'bucket',
		path
	}
});

describe('resolveExternalDrop', () => {
	it('maps dropped paths to local physical sources', () => {
		const decision = resolveExternalDrop(
			physicalFolder('/home/user/docs'),
			['/tmp/a.png', '/tmp/b.pdf']
		);

		expect(decision.kind).toBe('copy');
		if (decision.kind === 'copy') {
			expect(decision.destination).toEqual(
				physicalFolder('/home/user/docs')
			);
			expect(decision.sources).toHaveLength(2);
			expect(decision.sources[0]).toEqual({
				Physical: {device_slug: 'local', path: '/tmp/a.png'}
			});
			expect(decision.sources[1]).toEqual({
				Physical: {device_slug: 'local', path: '/tmp/b.pdf'}
			});
		}
	});

	it('accepts a destination addressed by the real device slug', () => {
		const decision = resolveExternalDrop(
			remoteDestination('/home/user/docs'),
			['/tmp/a.png']
		);

		expect(decision.kind).toBe('copy');
	});

	it('rejects a drop with no files', () => {
		const decision = resolveExternalDrop(physicalFolder('/home'), []);

		expect(decision).toEqual({kind: 'reject', reason: 'no-files'});
	});

	it('rejects a drop with no destination', () => {
		const decision = resolveExternalDrop(null, ['/tmp/a.png']);

		expect(decision).toEqual({kind: 'reject', reason: 'no-destination'});
	});

	it('rejects a drop into a non-physical destination', () => {
		for (const destination of [
			cloudDestination('reports'),
			{
				Content: {content_id: '00000000-0000-0000-0000-000000000000'}
			} satisfies SdPath
		]) {
			const decision = resolveExternalDrop(destination, ['/tmp/a.png']);

			expect(decision).toEqual({
				kind: 'reject',
				reason: 'not-physical'
			});
		}
	});
});
