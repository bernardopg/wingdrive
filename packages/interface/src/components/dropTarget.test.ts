import {describe, expect, it} from 'bun:test';
import type {File, WingPath} from '@wingdrive/ts-client';

import {alreadyIn, guessDropOperation, sameDevice} from './dropTarget';

const at = (device_slug: string, path: string): WingPath => ({Physical: {device_slug, path}});
const file = (wing_path: WingPath) => ({wing_path}) as File;

describe('drop target', () => {
	it('treats local and the real slug of this device as one device', () => {
		expect(sameDevice('local', 'archlinux', 'archlinux')).toBe(true);
		expect(sameDevice('archlinux', 'local', 'archlinux')).toBe(true);
		expect(sameDevice('local', 'phone', 'archlinux')).toBe(false);
		expect(sameDevice('local', 'archlinux', undefined)).toBe(false);
	});

	it('moves on the same device whichever slug each side carries', () => {
		expect(guessDropOperation(at('local', '/a/f'), at('archlinux', '/b'), 'archlinux')).toBe('move');
		expect(guessDropOperation(at('archlinux', '/a/f'), at('local', '/b'), 'archlinux')).toBe('move');
		expect(guessDropOperation(at('local', '/a/f'), at('phone', '/b'), 'archlinux')).toBe('copy');
		const cloud = {Cloud: {service: 's3', identifier: 'b', path: 'f'}} as unknown as WingPath;
		expect(guessDropOperation(cloud, at('local', '/b'), 'archlinux')).toBe('copy');
	});

	it('sees files already in the destination across slugs', () => {
		const files = [file(at('local', '/a/one')), file(at('local', '/a/two/'))];
		expect(alreadyIn(files, at('archlinux', '/a/'), 'archlinux')).toBe(true);
		expect(alreadyIn(files, at('archlinux', '/b'), 'archlinux')).toBe(false);
		expect(alreadyIn([file(at('local', '/top'))], at('local', '/'), 'archlinux')).toBe(true);
	});
});
