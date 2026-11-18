import { describe, expect, test } from "bun:test";
import { displayName } from "./defaultNames";

const t = (key: string) => `pt:${key}`;

describe("displayName", () => {
	test("translates the names the core seeds", () => {
		expect(displayName("My Library", t)).toBe("pt:My Library");
		expect(displayName("All Devices", t)).toBe("pt:All Devices");
	});

	test("keeps names the user typed, even when they match a catalog phrase", () => {
		expect(displayName("Trabalho", t)).toBe("Trabalho");
		expect(displayName("Settings", t)).toBe("Settings");
	});
});
