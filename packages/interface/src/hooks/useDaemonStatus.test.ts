import {expect, test} from 'bun:test';
import {patchStatus, type DaemonStatus} from './useDaemonStatus';

const connected: DaemonStatus = {
	isConnected: true,
	isChecking: false,
	isInstalled: false,
	isStarting: false,
	startupTimedOut: false
};

test('a poll that changes nothing keeps the same status object', () => {
	expect(patchStatus(connected, {isConnected: true, isChecking: false, isStarting: false})).toBe(connected);
});

test('a changed field makes a new status and leaves the old one alone', () => {
	const next = patchStatus(connected, {isConnected: false});
	expect(next).not.toBe(connected);
	expect(next).toEqual({...connected, isConnected: false});
	expect(connected.isConnected).toBe(true);
});
