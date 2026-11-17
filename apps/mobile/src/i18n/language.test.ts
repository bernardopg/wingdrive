import { describe, expect, test } from "bun:test";
import { languageForLocale, parseLanguageChoice, resolveLanguage } from "./language";

describe("languageForLocale", () => {
	test("maps every Portuguese locale to pt", () => {
		expect(languageForLocale("pt-BR")).toBe("pt");
		expect(languageForLocale("pt_PT")).toBe("pt");
		expect(languageForLocale("PT")).toBe("pt");
	});

	test("falls back to English", () => {
		expect(languageForLocale("es-ES")).toBe("en");
		expect(languageForLocale(undefined)).toBe("en");
		expect(languageForLocale("")).toBe("en");
	});
});

describe("parseLanguageChoice", () => {
	test("keeps known choices and drops the rest", () => {
		expect(parseLanguageChoice("system")).toBe("system");
		expect(parseLanguageChoice("pt")).toBe("pt");
		expect(parseLanguageChoice("fr")).toBeNull();
		expect(parseLanguageChoice(null)).toBeNull();
	});
});

describe("resolveLanguage", () => {
	test("a fresh install follows the phone", () => {
		expect(resolveLanguage(null, "en", "pt-BR")).toBe("pt");
		expect(resolveLanguage(null, undefined, "en-US")).toBe("en");
	});

	test("a choice made in Settings wins over the phone", () => {
		expect(resolveLanguage("en", "en", "pt-BR")).toBe("en");
		expect(resolveLanguage("pt", "en", "en-US")).toBe("pt");
	});

	test("the system choice follows the phone even when the core says pt", () => {
		expect(resolveLanguage("system", "pt", "en-US")).toBe("en");
	});

	test("Portuguese picked before the choice was stored is kept", () => {
		expect(resolveLanguage(null, "pt", "en-US")).toBe("pt");
	});
});
