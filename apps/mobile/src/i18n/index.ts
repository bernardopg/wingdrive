import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import en from './locales/en/common.json';
import pt from './locales/pt/common.json';

void i18n.use(initReactI18next).init({
	resources: { en: { common: en }, pt: { common: pt } },
	defaultNS: 'common',
	lng: 'en',
	fallbackLng: 'en',
	// Keys are the English phrases themselves, which may contain '.' or ':'.
	keySeparator: false,
	nsSeparator: false,
	interpolation: { escapeValue: false },
});

export function applyLanguage(language?: string | null) {
	void i18n.changeLanguage(language === 'pt' ? 'pt' : 'en');
}

export default i18n;
