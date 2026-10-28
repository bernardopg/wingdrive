import type {VcsFileStatus} from '@wingdrive/ts-client';
import clsx from 'clsx';

import {useTranslation} from '../../../i18n';

// Letters and colors follow the convention editors use for Git state.
const EMBLEMS: Record<VcsFileStatus, {letter: string; className: string}> = {
	untracked: {letter: 'U', className: 'bg-ink-faint text-app'},
	added: {letter: 'A', className: 'bg-status-success text-white'},
	modified: {letter: 'M', className: 'bg-status-warning text-black'},
	conflicted: {letter: 'C', className: 'bg-status-error text-white'}
};

interface VcsEmblemProps {
	status: VcsFileStatus;
	size?: 'sm' | 'md';
	className?: string;
}

/** Git state badge drawn over a file's thumbnail. */
export function VcsEmblem({status, size = 'md', className}: VcsEmblemProps) {
	const {t} = useTranslation('explorer');
	const emblem = EMBLEMS[status];
	const label = t(`vcs.${status}`);

	return (
		<span
			role="img"
			aria-label={label}
			title={label}
			className={clsx(
				'pointer-events-auto flex items-center justify-center rounded-full font-bold leading-none ring-2 ring-app',
				size === 'md' ? 'size-4 text-[9px]' : 'size-3 text-[7px]',
				emblem.className,
				className
			)}
		>
			{emblem.letter}
		</span>
	);
}
