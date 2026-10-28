import clsx from 'clsx';
import {useCallback} from 'react';

import {uiLocale, useTranslation} from '../../../i18n';
import {SEARCH_SIZE_PRESETS} from '../hooks/searchQuery';
import type {GroupLabel} from './grouping';

/** Height of a section header row, shared by the grid and list virtualizers. */
export const GROUP_HEADER_HEIGHT = 32;

/** Translates a section label into the text its header shows. */
export function useGroupLabel() {
	const {t} = useTranslation(['explorer', 'inspector']);

	return useCallback(
		(label: GroupLabel): string => {
			switch (label.type) {
				case 'folders':
					return t('explorer:groups.folders');
				case 'letter':
					return label.letter;
				case 'other':
					return t('explorer:groups.other');
				case 'date':
					return t(`explorer:groups.${label.bucket}`);
				case 'month': {
					const text = new Intl.DateTimeFormat(uiLocale(), {
						month: 'long',
						year: 'numeric'
					}).format(new Date(label.year, label.month, 1));
					return text.charAt(0).toLocaleUpperCase() + text.slice(1);
				}
				case 'year':
					return String(label.year);
				case 'size': {
					const preset = SEARCH_SIZE_PRESETS.find((p) => p.preset === label.preset);
					return preset ? t(`explorer:${preset.label}`) : label.preset;
				}
				case 'kind':
					return t(`inspector:kinds.${label.kind}`, {defaultValue: label.kind});
				case 'tag':
					return label.name;
				case 'untagged':
					return t('explorer:groups.untagged');
			}
		},
		[t]
	);
}

interface GroupHeaderProps {
	label: string;
	count: number;
	className?: string;
	style?: React.CSSProperties;
}

/** Section title with its item count, as Dolphin and Finder draw groups. */
export function GroupHeader({label, count, className, style}: GroupHeaderProps) {
	return (
		<div
			role="heading"
			aria-level={3}
			className={clsx('flex items-end gap-2 pb-1.5 text-xs font-semibold', className)}
			style={{height: GROUP_HEADER_HEIGHT, ...style}}
		>
			<span className="text-ink truncate">{label}</span>
			<span className="text-ink-faint font-normal">{count}</span>
			<div className="bg-app-line mb-1.5 h-px flex-1" />
		</div>
	);
}
