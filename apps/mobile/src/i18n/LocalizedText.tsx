import { Children } from 'react';
import { Text as NativeText, type TextProps } from 'react-native';
import { useTranslation } from 'react-i18next';

export function LocalizedText({ children, ...props }: TextProps) {
	const { t, i18n } = useTranslation('common');

	return (
		<NativeText {...props}>
			{Children.map(children, child => {
				if (typeof child !== 'string') return child;
				const value = child.trim();
				// Dynamic text such as file and device names is shown as-is.
				if (!value || !i18n.exists(value)) return child;
				const start = child.indexOf(value);
				return `${child.slice(0, start)}${t(value)}${child.slice(start + value.length)}`;
			})}
		</NativeText>
	);
}
