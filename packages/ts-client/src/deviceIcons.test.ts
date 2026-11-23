import {expect, test} from 'bun:test';
import {getDeviceIcon, getDeviceIconBySlug} from './deviceIcons';

const base = (overrides: Record<string, unknown> = {}) =>
	({
		name: 'Device',
		os: 'Linux',
		hardware_model: null,
		...overrides
	}) as any;

test('getDeviceIcon prefers hardware model matches', () => {
	for (const hardware_model of [
		'Mac14,3', 'Mac14,13', 'iPad Pro', 'iPhone 15',
		'MacBook Pro', 'iMac', 'Surface Tablet', 'Android Phone'
	]) {
		expect(typeof getDeviceIcon(base({hardware_model}))).toBe('string');
	}
});

test('getDeviceIcon falls back to OS', () => {
	expect(typeof getDeviceIcon(base({os: 'iOS'}))).toBe('string');
	expect(typeof getDeviceIcon(base({os: 'android'}))).toBe('string');
	expect(typeof getDeviceIcon(base({os: 'macOS'}))).toBe('string');
	expect(typeof getDeviceIcon(base({os: 'Windows'}))).toBe('string');
	expect(typeof getDeviceIcon(base({os: 'Linux', name: 'My Server'}))).toBe('string');
	expect(typeof getDeviceIcon(base({os: 'Linux', name: 'Laptop'}))).toBe('string');
	expect(typeof getDeviceIcon(base({os: 'UnknownOS'}))).toBe('string');
});

test('getDeviceIconBySlug returns default for missing device', () => {
	const devices = new Map([['a', base({os: 'Windows'})]]);
	expect(typeof getDeviceIconBySlug('missing', devices)).toBe('string');
	expect(typeof getDeviceIconBySlug('a', devices)).toBe('string');
});
