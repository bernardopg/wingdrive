import { describe, expect, test } from "bun:test";
import { describeDaemonError } from "./client";

describe("describeDaemonError", () => {
	test("uses the variant's message", () => {
		expect(
			describeDaemonError({ OperationFailed: "Invalid input: theme: bad" }),
		).toBe("Invalid input: theme: bad");
	});
	test("names unit variants and keeps unknown shapes", () => {
		expect(describeDaemonError({ Timeout: null })).toBe("Timeout");
		expect(describeDaemonError("plain")).toBe("plain");
		expect(describeDaemonError({ a: 1, b: 2 })).toBe('{"a":1,"b":2}');
	});
});
