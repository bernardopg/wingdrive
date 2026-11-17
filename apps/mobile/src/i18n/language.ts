// Which language the app shows. Kept free of React Native imports so bun can test it.

export type Language = "en" | "pt";
export type LanguageChoice = Language | "system";

/** AsyncStorage key of the language picked in Settings: "en", "pt" or "system". */
export const LANGUAGE_CHOICE_KEY = "wingdrive-language";

/** The supported language closest to a BCP 47 locale such as "pt-BR" or "pt_BR". */
export function languageForLocale(locale: string | null | undefined): Language {
	return locale?.toLowerCase().startsWith("pt") ? "pt" : "en";
}

/** Parses a stored choice; anything unknown counts as no choice. */
export function parseLanguageChoice(value: string | null | undefined): LanguageChoice | null {
	return value === "en" || value === "pt" || value === "system" ? value : null;
}

/**
 * The language to show.
 *
 * A choice made in Settings wins. With no choice, the phone's language is used, except that a
 * core config already set to "pt" is kept: the core defaults to "en", so "pt" there was picked
 * by the user before the choice was stored on the phone.
 */
export function resolveLanguage(
	choice: LanguageChoice | null,
	coreLanguage: string | null | undefined,
	systemLocale: string | null | undefined,
): Language {
	if (choice === "en" || choice === "pt") return choice;
	if (choice === null && coreLanguage === "pt") return "pt";
	return languageForLocale(systemLocale);
}
