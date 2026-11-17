import { describe, expect, test } from "bun:test";
import { fileSystemLabel, storageTotals, volumeTypeLabel } from "./volumeLabels";

describe("fileSystemLabel", () => {
	test("hides Android's placeholder and unknown values", () => {
		expect(fileSystemLabel({ Other: "android" })).toBeNull();
		expect(fileSystemLabel("Unknown")).toBeNull();
		expect(fileSystemLabel(null)).toBeNull();
	});

	test("keeps real file systems", () => {
		expect(fileSystemLabel("ExFAT")).toBe("ExFAT");
		expect(fileSystemLabel({ Other: "f2fs" })).toBe("f2fs");
	});
});

describe("volumeTypeLabel", () => {
	test("names volume types in words the user knows", () => {
		expect(volumeTypeLabel("Primary")).toBe("Internal");
		expect(volumeTypeLabel("External")).toBe("External");
	});

	test("hides types with nothing to say", () => {
		expect(volumeTypeLabel("Unknown")).toBeNull();
		expect(volumeTypeLabel("System")).toBeNull();
		expect(volumeTypeLabel({ Other: "weird" })).toBeNull();
	});
});

describe("storageTotals", () => {
	const phone = {
		fingerprint: "a",
		volume_type: "Primary",
		total_capacity: 50,
		available_space: 46,
	};

	test("keeps the library's totals when it has any", () => {
		expect(storageTotals({ total: 100, available: 10 }, [phone])).toEqual({ total: 100, available: 10 });
	});

	test("counts detected volumes when the library tracks none", () => {
		const card = { ...phone, fingerprint: "b", volume_type: "External", total_capacity: 30, available_space: 20 };
		expect(storageTotals({ total: 0, available: 0 }, [phone, card])).toEqual({ total: 80, available: 66 });
	});

	test("skips duplicates, hidden and system volumes", () => {
		const volumes = [
			phone,
			{ ...phone },
			{ ...phone, fingerprint: "c", is_user_visible: false },
			{ ...phone, fingerprint: "d", volume_type: "System" },
		];
		expect(storageTotals({ total: 0, available: 0 }, volumes)).toEqual({ total: 50, available: 46 });
	});

	test("stays at zero without volumes", () => {
		expect(storageTotals({ total: 0, available: 0 }, undefined)).toEqual({ total: 0, available: 0 });
	});
});
