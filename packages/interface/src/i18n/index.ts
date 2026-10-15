import i18n from 'i18next';
import {initReactI18next, useTranslation} from 'react-i18next';
import settingsEn from './locales/en/settings.json';
import settingsPt from './locales/pt/settings.json';
import explorerEn from './locales/en/explorer.json';
import explorerPt from './locales/pt/explorer.json';
import inspectorEn from './locales/en/inspector.json';
import inspectorPt from './locales/pt/inspector.json';
import dialogsEn from './locales/en/dialogs.json';
import dialogsPt from './locales/pt/dialogs.json';
import navigationEn from './locales/en/navigation.json';
import navigationPt from './locales/pt/navigation.json';
import commonEn from './locales/en/common.json';
import commonPt from './locales/pt/common.json';

/**
 * UI languages. Ids are ISO 639-1 codes because the daemon stores
 * `preferences.language` as a two-letter code; Crowdin writes each
 * translation to `locales/<two_letters_code>/`.
 */
export const LANGUAGES = [
	{id: 'en', name: 'English'},
	{id: 'pt', name: 'Português (Brasil)'}
] as const;

void i18n.use(initReactI18next).init({
	resources: {
		en: {settings: settingsEn, explorer: explorerEn, inspector: inspectorEn, dialogs: dialogsEn, navigation: navigationEn, common: commonEn},
		pt: {settings: settingsPt, explorer: explorerPt, inspector: inspectorPt, dialogs: dialogsPt, navigation: navigationPt, common: commonPt}
	},
	ns: ['settings', 'explorer', 'inspector', 'dialogs', 'navigation', 'common'],
	lng: 'en',
	fallbackLng: 'en',
	defaultNS: 'settings',
	interpolation: {escapeValue: false}
});

export function applyLanguage(language: string) {
	const id = LANGUAGES.some((l) => l.id === language) ? language : 'en';
	document.documentElement.lang = id === 'pt' ? 'pt-BR' : id;
	void i18n.changeLanguage(id);
}

export {i18n, useTranslation};

/** BCP 47 tag for Intl/date formatting in the current UI language. */
export function uiLocale() {
	return i18n.language === 'pt' ? 'pt-BR' : 'en-US';
}
