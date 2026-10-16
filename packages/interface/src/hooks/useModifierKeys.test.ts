import { expect, test } from "bun:test";
import { dropOperation, type ModifierKeys } from "./useModifierKeys";

const keys = (held: Partial<ModifierKeys>): ModifierKeys => ({
	alt: false,
	shift: false,
	meta: false,
	ctrl: false,
	...held,
});

test("Ctrl forces copy, Shift forces move, otherwise the guess stands", () => {
	expect(dropOperation("move", keys({ ctrl: true }))).toBe("copy");
	expect(dropOperation("copy", keys({ shift: true }))).toBe("move");
	expect(dropOperation("move", keys({}))).toBe("move");
	expect(dropOperation("copy", keys({}))).toBe("copy");
	expect(dropOperation("copy", keys({ ctrl: true, shift: true }))).toBe("copy");
	expect(dropOperation("move", keys({ alt: true }))).toBe("move");
});
