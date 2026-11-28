import { describe, expect, test } from "bun:test";
import type { BackgroundSyncState } from "wing-mobile-core";
import { backgroundSyncHints, needsNotificationPrompt } from "./backgroundSync";

const on: BackgroundSyncState = {
	supported: true,
	enabled: true,
	running: true,
	unrestrictedBattery: true,
	notificationsAllowed: true,
};

describe("backgroundSyncHints", () => {
	test("asks nothing when everything is allowed", () => {
		expect(backgroundSyncHints(on)).toEqual([]);
	});

	test("asks for battery and notifications while they are restricted", () => {
		expect(
			backgroundSyncHints({ ...on, unrestrictedBattery: false, notificationsAllowed: false }),
		).toEqual(["battery", "notifications"]);
	});

	test("asks nothing while background sync is off or unsupported", () => {
		const restricted = { ...on, unrestrictedBattery: false, notificationsAllowed: false };
		expect(backgroundSyncHints({ ...restricted, enabled: false })).toEqual([]);
		expect(backgroundSyncHints({ ...restricted, supported: false })).toEqual([]);
	});
});

describe("needsNotificationPrompt", () => {
	test("prompts on Android 13 and later only", () => {
		expect(needsNotificationPrompt("android", 33)).toBe(true);
		expect(needsNotificationPrompt("android", 36)).toBe(true);
		expect(needsNotificationPrompt("android", 32)).toBe(false);
		expect(needsNotificationPrompt("ios", 36)).toBe(false);
	});
});
