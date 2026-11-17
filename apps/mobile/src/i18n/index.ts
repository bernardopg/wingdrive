import AsyncStorage from '@react-native-async-storage/async-storage';
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import {
	LANGUAGE_CHOICE_KEY,
	type Language,
	type LanguageChoice,
	languageForLocale,
	parseLanguageChoice,
	resolveLanguage,
} from './language';
import en from './locales/en/common.json';
import pt from './locales/pt/common.json';

void i18n.use(initReactI18next).init({
	resources: { en: { common: en }, pt: { common: pt } },
	defaultNS: 'common',
	// Until the core answers, show the phone's language rather than English (MOB-017).
	lng: languageForLocale(systemLocale()),
	fallbackLng: 'en',
	// Keys are the English phrases themselves, which may contain '.' or ':'.
	keySeparator: false,
	nsSeparator: false,
	interpolation: { escapeValue: false },
});

/** The phone's locale, such as "pt-BR"; Hermes reads it from the system. */
export function systemLocale(): string | undefined {
	try {
		return Intl.DateTimeFormat().resolvedOptions().locale;
	} catch {
		return undefined;
	}
}

export function applyLanguage(language?: string | null) {
	void i18n.changeLanguage(language === 'pt' ? 'pt' : 'en');
}

/** The language picked in Settings, or null when the user never picked one. */
export async function loadLanguageChoice(): Promise<LanguageChoice | null> {
	try {
		return parseLanguageChoice(await AsyncStorage.getItem(LANGUAGE_CHOICE_KEY));
	} catch {
		return null;
	}
}

/** Shows the language the user picked, or the phone's when they never picked one. */
export async function applyStartupLanguage(coreLanguage?: string | null): Promise<Language> {
	const language = resolveLanguage(await loadLanguageChoice(), coreLanguage, systemLocale());
	applyLanguage(language);
	return language;
}

/** Stores a choice made in Settings and shows it. */
export async function chooseLanguage(choice: LanguageChoice): Promise<Language> {
	const language = choice === 'system' ? languageForLocale(systemLocale()) : choice;
	applyLanguage(language);
	try {
		await AsyncStorage.setItem(LANGUAGE_CHOICE_KEY, choice);
	} catch {
		// The language still changes for this session.
	}
	return language;
}

export default i18n;
