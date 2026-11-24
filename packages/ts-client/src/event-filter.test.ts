import { describe, expect, it } from "bun:test";
import { DEFAULT_EVENT_SUBSCRIPTION, NOISY_EVENTS } from "./event-filter";

describe("event-filter", () => {
	it("exposes the default subscription list used by the desktop shell", () => {
		expect(DEFAULT_EVENT_SUBSCRIPTION).toContain("CoreStarted");
		expect(DEFAULT_EVENT_SUBSCRIPTION).toContain("ResourceChanged");
		expect(DEFAULT_EVENT_SUBSCRIPTION).toContain("ResourceChangedBatch");
		expect(DEFAULT_EVENT_SUBSCRIPTION).toContain("ResourceDeleted");
		expect(DEFAULT_EVENT_SUBSCRIPTION).toContain("JobProgress");
		expect(DEFAULT_EVENT_SUBSCRIPTION).toContain("WingdropUpdated");
	});

	it("does not include noisy events in the default subscription", () => {
		for (const event of NOISY_EVENTS) {
			expect(DEFAULT_EVENT_SUBSCRIPTION).not.toContain(event);
		}
	});

	it("marks indexing progress as noisy", () => {
		expect(NOISY_EVENTS).toEqual(["IndexingProgress"]);
	});

	it("contains no duplicates", () => {
		const unique = new Set(DEFAULT_EVENT_SUBSCRIPTION);
		expect(unique.size).toBe(DEFAULT_EVENT_SUBSCRIPTION.length);
	});
});
